//! Main Facebook video extractor implementation

use crate::{
    common::{
        error::{FacebookExtractorError, Result},
        types::{VideoInfo, VideoQuality, FacebookContentType, PrivacyLevel, AccessMethod, UrlValidation, StreamType, AspectRatio},
        config::ExtractorConfig,
        utils::UrlCleaner,
    },
    extraction::{metadata::MetadataExtractor, streams::StreamAnalyzer},
    USER_AGENTS,
};
use reqwest::Client;
use regex::Regex;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn, error, instrument};

/// Main Facebook video extractor
pub struct FacebookExtractor {
    client: Client,
    config: ExtractorConfig,
    cache: HashMap<String, VideoInfo>,
    metadata_extractor: MetadataExtractor,
    stream_analyzer: StreamAnalyzer,
    url_cleaner: UrlCleaner,
}

impl FacebookExtractor {
    /// Create a new extractor with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ExtractorConfig::default())
    }

    /// Create a new extractor with custom configuration
    pub fn with_config(config: ExtractorConfig) -> Result<Self> {
        let client = Self::build_http_client(&config)?;

        Ok(Self {
            client,
            config,
            cache: HashMap::new(),
            metadata_extractor: MetadataExtractor::new(),
            stream_analyzer: StreamAnalyzer::new(),
            url_cleaner: UrlCleaner::new(),
        })
    }

    /// Build HTTP client with configuration
    fn build_http_client(config: &ExtractorConfig) -> Result<Client> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.network.timeout_secs))
            .connect_timeout(Duration::from_secs(config.network.connection_timeout_secs))
            .pool_idle_timeout(Duration::from_secs(config.network.pool_idle_timeout_secs))
            .pool_max_idle_per_host(config.network.pool_max_idle_per_host)
            .tcp_keepalive(Duration::from_secs(config.network.tcp_keepalive_secs))
            .redirect(reqwest::redirect::Policy::limited(config.network.max_redirects))
            .build()
            .map_err(|e| FacebookExtractorError::Config {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(client)
    }

    /// Get primary user agent for requests
    fn primary_user_agent(&self) -> &str {
        self.config
            .network
            .user_agents
            .first()
            .map(|s| s.as_str())
            .unwrap_or(USER_AGENTS[0])
    }

    /// Validate Facebook URL and determine content type
    pub fn validate_url(&self, url: &str) -> UrlValidation {
        if !self.is_valid_facebook_url(url) {
            return UrlValidation::invalid(
                "Not a valid Facebook URL".to_string(),
                vec![
                    "URL must contain 'facebook.com'".to_string(),
                    "Supported formats: /watch?v=ID, /videos/ID, /reel/ID, /share/v/ID".to_string(),
                ],
            );
        }

        // Determine content type
        let content_type = if url.contains("/reel/") || url.contains("/share/r/") {
            FacebookContentType::Reel
        } else if url.contains("/story/") {
            FacebookContentType::Story
        } else if url.contains("/live/") {
            FacebookContentType::LiveVideo
        } else {
            FacebookContentType::RegularVideo
        };

        // Extract video ID
        match self.extract_video_id(url) {
            Ok(video_id) => UrlValidation::valid(content_type, video_id),
            Err(e) => UrlValidation::invalid(
                format!("Could not extract video ID: {}", e),
                vec![
                    "Check if the URL is complete and valid".to_string(),
                    "Try copying the URL directly from Facebook".to_string(),
                ],
            ),
        }
    }

    /// Check if URL is a valid Facebook URL
    pub fn is_valid_facebook_url(&self, url: &str) -> bool {
        url.contains("facebook.com") &&
        (url.contains("/watch") || url.contains("/videos/") || url.contains("/reel/") || url.contains("/share/"))
    }

    /// Extract video ID from Facebook URL using regex patterns
    pub fn extract_video_id(&self, url: &str) -> Result<String> {
        let patterns = [
            r"facebook\.com/watch/?\?.*[&?]v=(\d+)",  // v= parameter after other params
            r"facebook\.com/watch/?\?v=(\d+)",        // v= parameter as first param
            r"facebook\.com/.*/videos/(\d+)",
            r"facebook\.com/reel/(\d+)",
            r"fb\.watch/([a-zA-Z0-9]+)",              // Short URLs
            r"facebook\.com/share/v/([a-zA-Z0-9_-]+)", // New sharing format for videos
            r"facebook\.com/share/r/([a-zA-Z0-9_-]+)", // New sharing format for reels
        ];

        for pattern in &patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(url) {
                    if let Some(id_match) = capture.get(1) {
                        return Ok(id_match.as_str().to_string());
                    }
                }
            }
        }

        Err(FacebookExtractorError::video_id_extraction(
            "Could not extract video ID from URL",
        ))
    }

    /// Check if URL is a Facebook share URL that needs transformation
    pub fn is_share_url(&self, url: &str) -> bool {
        url.contains("/share/v/") || url.contains("/share/r/")
    }

    /// Transform Facebook share URL to a more accessible format
    pub async fn transform_share_url(&self, url: &str) -> Result<String> {
        if !self.is_share_url(url) {
            return Ok(url.to_string());
        }

        info!("Attempting to transform share URL: {}", url);

        // Try to follow redirects to get the actual video URL
        let client = &self.client;

        // Use multiple strategies to transform the URL
        let strategies = vec![
            self.try_redirect_follow(url, &client).await,
            self.try_mobile_redirect(url, &client).await,
            self.try_oembed_transform(url, &client).await,
        ];

        for strategy_result in strategies {
            if let Ok(transformed_url) = strategy_result {
                if transformed_url != url && !self.is_share_url(&transformed_url) {
                    info!("Successfully transformed share URL to: {}", transformed_url);
                    return Ok(transformed_url);
                }
            }
        }

        warn!("Could not transform share URL, using original: {}", url);
        Ok(url.to_string())
    }

    /// Try to follow redirects to get actual video URL
    async fn try_redirect_follow(&self, url: &str, client: &reqwest::Client) -> Result<String> {
        let response = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .send()
            .await?;

        let final_url = response.url().to_string();
        if final_url != url {
            return Ok(final_url);
        }

        Err(FacebookExtractorError::html_parsing("No redirect found"))
    }

    /// Try mobile redirect approach
    async fn try_mobile_redirect(&self, url: &str, client: &reqwest::Client) -> Result<String> {
        let mobile_url = url.replace("www.facebook.com", "m.facebook.com");

        let response = client
            .get(&mobile_url)
            .header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .send()
            .await?;

        let final_url = response.url().to_string();
        if final_url != mobile_url && !self.is_share_url(&final_url) {
            return Ok(final_url.replace("m.facebook.com", "www.facebook.com"));
        }

        Err(FacebookExtractorError::html_parsing("No mobile redirect found"))
    }

    /// Try oEmbed API to get video information
    async fn try_oembed_transform(&self, url: &str, client: &reqwest::Client) -> Result<String> {
        let encoded_url = urlencoding::encode(url);
        let oembed_url = format!("https://www.facebook.com/plugins/video/oembed.json/?url={}", encoded_url);

        let response = client
            .get(&oembed_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await?;

        if response.status().is_success() {
            let text = response.text().await?;
            // Try to extract video URL from oEmbed response
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(html) = json.get("html").and_then(|h| h.as_str()) {
                    // Look for video URL in the embed HTML
                    if let Some(video_url) = self.extract_video_url_from_embed_html(html) {
                        return Ok(video_url);
                    }
                }
            }
        }

        Err(FacebookExtractorError::html_parsing("oEmbed transformation failed"))
    }

    /// Extract video URL from embed HTML
    fn extract_video_url_from_embed_html(&self, html: &str) -> Option<String> {
        let patterns = vec![
            r#"href="([^"]*facebook\.com[^"]*videos[^"]*)"#,
            r#"src="([^"]*facebook\.com[^"]*videos[^"]*)"#,
            r#"data-href="([^"]*facebook\.com[^"]*videos[^"]*)"#,
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(url_match) = capture.get(1) {
                        let url = url_match.as_str().replace("&amp;", "&");
                        if !self.is_share_url(&url) {
                            return Some(url);
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract video information from Facebook URL
    #[instrument(skip(self), fields(url = %url))]
    pub async fn extract_video_info(&self, url: &str) -> Result<VideoInfo> {
        info!("Starting video extraction");

        // Transform share URLs if needed
        let working_url = if self.is_share_url(url) {
            info!("Detected share URL, attempting transformation");
            match self.transform_share_url(url).await {
                Ok(transformed_url) => {
                    info!("Share URL transformed to: {}", transformed_url);
                    transformed_url
                }
                Err(e) => {
                    warn!("Share URL transformation failed: {}, using original", e);
                    url.to_string()
                }
            }
        } else {
            url.to_string()
        };

        // Validate URL (use working_url for validation but keep original for metadata)
        let validation = self.validate_url(&working_url);
        if !validation.is_valid {
            error!("Invalid URL provided");
            return Err(FacebookExtractorError::invalid_url(url));
        }

        let video_id = validation.video_id.unwrap();
        let content_type = validation.content_type.unwrap();

        info!(video_id = %video_id, content_type = ?content_type, "URL validation passed");

        // Check cache first (use original URL for cache key)
        if let Some(cached_info) = self.get_cached_info(url) {
            info!("Using cached video info");
            return Ok(cached_info.clone());
        }

        // Try multiple extraction methods (use working_url for extraction)
        let mut _last_error = None;

        // For reels, prioritize reel-specific extraction first
        if content_type == FacebookContentType::Reel {
            info!("Trying reel-specific extraction (priority for reels)");
            match self.try_reel_extraction(&working_url).await {
                Ok(mut info) => {
                    info!("Reel extraction succeeded");
                    info.video_id = video_id;
                    info.source_url = url.to_string();
                    info.content_type = content_type;
                    info.access_method = AccessMethod::Mobile;
                    return Ok(info);
                }
                Err(e) => {
                    warn!("Reel extraction failed: {}", e);
                    _last_error = Some(e);
                }
            }
        }

        // Method 1: Direct fetch with spoofed headers
        info!("Trying direct fetch with spoofed headers");
        match self.try_direct_fetch(&working_url).await {
            Ok(mut info) => {
                info!("Direct fetch succeeded");
                info.video_id = video_id;
                info.source_url = url.to_string(); // Keep original URL in metadata
                info.content_type = content_type;
                info.access_method = AccessMethod::Direct;
                return Ok(info);
            }
            Err(e) => {
                warn!("Direct fetch failed: {}", e);
                _last_error = Some(e);
            }
        }

        // Method 2: Mobile version
        info!("Trying mobile version");
        match self.try_mobile_version(&working_url).await {
            Ok(mut info) => {
                info!("Mobile version succeeded");
                info.video_id = video_id;
                info.source_url = url.to_string(); // Keep original URL in metadata
                info.content_type = content_type;
                info.access_method = AccessMethod::Mobile;
                return Ok(info);
            }
            Err(e) => {
                warn!("Mobile version failed: {}", e);
                _last_error = Some(e);
            }
        }

        // Method 3: Alternative public extraction (bypass auth checks)
        info!("Trying alternative public extraction methods");
        match self.try_public_extraction(&working_url).await {
            Ok(mut info) => {
                info!("Alternative public extraction succeeded");
                info.video_id = video_id;
                info.source_url = url.to_string(); // Keep original URL in metadata
                info.content_type = content_type;
                info.access_method = AccessMethod::Alternative;
                return Ok(info);
            }
            Err(e) => {
                warn!("Alternative public extraction failed: {}", e);
                _last_error = Some(e);
            }
        }

        // Method 4: Reel-specific extraction (fallback for non-reels)
        if content_type == FacebookContentType::Reel {
            info!("Trying reel-specific extraction (fallback)");
            match self.try_reel_extraction(&working_url).await {
                Ok(mut info) => {
                    info!("Reel extraction succeeded");
                    info.video_id = video_id;
                    info.source_url = url.to_string(); // Keep original URL in metadata
                    info.content_type = content_type;
                    info.access_method = AccessMethod::Mobile;
                    return Ok(info);
                }
                Err(e) => {
                    warn!("Reel extraction failed: {}", e);
                    _last_error = Some(e);
                }
            }
        }

        error!("All extraction methods failed");
        Err(_last_error.unwrap_or_else(|| {
            FacebookExtractorError::html_parsing("All extraction methods failed")
        }))
    }

    /// Extract from view-source content (for private videos)
    pub async fn extract_from_view_source(&self, content: &str) -> Result<VideoInfo> {
        info!("Extracting from view-source content");

        // Clean view-source content
        let cleaned_html = self.clean_view_source_content(content)?;

        // Try to extract video ID from the content
        let video_id = self.extract_video_id_from_html(&cleaned_html)
            .unwrap_or_else(|| "unknown".to_string());

        // Parse the HTML content
        let mut video_info = self.parse_facebook_html(&cleaned_html, "view-source").await?;
        video_info.video_id = video_id;
        video_info.content_type = FacebookContentType::PrivateVideo;
        video_info.privacy_level = PrivacyLevel::Private;
        video_info.access_method = AccessMethod::ViewSource;

        Ok(video_info)
    }

    /// Extract Reel information with specialized handling
    pub async fn extract_reel_info(&self, url: &str) -> Result<VideoInfo> {
        info!("Extracting reel information");

        // Validate it's a reel URL
        if !url.contains("/reel/") {
            return Err(FacebookExtractorError::invalid_url(url));
        }

        // Try reel-specific extraction
        self.try_reel_extraction(url).await
    }

    /// Create cache key for URL
    fn cache_key(&self, url: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        format!("video_{}", hasher.finish())
    }

    /// Get cached video info if available and not expired
    fn get_cached_info(&self, url: &str) -> Option<&VideoInfo> {
        if !self.config.caching.enable_caching {
            return None;
        }

        let cache_key = self.cache_key(url);
        self.cache.get(&cache_key).and_then(|info| {
            // Check if cache is still valid
            let age = chrono::Utc::now().signed_duration_since(info.extraction_timestamp);
            if age.num_hours() < self.config.caching.cache_expiry_hours as i64 {
                Some(info)
            } else {
                None
            }
        })
    }

    /// Try direct fetch with spoofed browser headers
    async fn try_direct_fetch(&self, url: &str) -> Result<VideoInfo> {
        info!("Fetching with enhanced browser headers");

        let response = self
            .client
            .get(url)
            .header("User-Agent", self.primary_user_agent())
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
            )
            .header("Accept-Language", "en-US,en;q=0.9")
            // NOTE: Removed Accept-Encoding to avoid compression issues with Facebook
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
            .header("Sec-Fetch-User", "?1")
            .header("Cache-Control", "max-age=0")
            .header("sec-ch-ua", "\"Google Chrome\";v=\"120\", \"Chromium\";v=\"120\", \"Not_A Brand\";v=\"99\"")
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"macOS\"")
            // Additional headers to better mimic a real browser
            .header("sec-ch-ua-full-version-list", "\"Google Chrome\";v=\"120.0.6099.109\", \"Chromium\";v=\"120.0.6099.109\", \"Not_A Brand\";v=\"99.0.0.0\"")
            .header("sec-ch-ua-arch", "\"x86\"")
            .header("sec-ch-ua-bitness", "\"64\"")
            .header("sec-ch-ua-model", "\"\"")
            .header("Viewport-Width", "1920")
            .header("Device-Memory", "8")
            .send()
            .await?;

        debug!("Response status: {}", response.status());

        if !response.status().is_success() {
            return Err(FacebookExtractorError::html_parsing(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        // Handle decompression properly
        let html = match response.text().await {
            Ok(text) => {
                debug!("Response HTML length: {} characters", text.len());
                // Check if the text looks like valid HTML/text
                if text.len() > 100 && (text.contains("<html") || text.contains("<!DOCTYPE") || text.contains("<head")) {
                    text
                } else if text.len() < 100 || text.chars().any(|c| c as u32 > 127 && !c.is_whitespace()) {
                    warn!("Response appears to be binary or corrupted, attempting manual decompression");
                    return Err(FacebookExtractorError::html_parsing(
                        "Response appears to be compressed binary data that wasn't properly decompressed"
                    ));
                } else {
                    text
                }
            }
            Err(e) => {
                error!("Failed to decode response as text: {}", e);
                return Err(FacebookExtractorError::html_parsing(
                    format!("Failed to decode response as text: {}", e)
                ));
            }
        };

        self.parse_facebook_html(&html, url).await
    }

    /// Try mobile version of Facebook
    async fn try_mobile_version(&self, url: &str) -> Result<VideoInfo> {
        info!("Trying mobile version");
        let mobile_url = url.replace("www.facebook.com", "m.facebook.com");
        self.try_direct_fetch(&mobile_url).await
    }

    /// Try alternative public extraction methods that bypass authentication
    async fn try_public_extraction(&self, url: &str) -> Result<VideoInfo> {
        info!("Trying alternative public extraction methods");

        // Method 1: Try with minimal headers to avoid triggering auth
        let minimal_response = self
            .client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (compatible; FacebookVideoBot/1.0)")
            .header("Accept", "text/html")
            .timeout(Duration::from_secs(15))
            .send()
            .await?;

        if minimal_response.status().is_success() {
            let html = minimal_response.text().await?;
            if let Ok(info) = self.parse_facebook_html_bypass_auth(&html, url).await {
                info!("Minimal headers extraction succeeded");
                return Ok(info);
            }
        }

        // Method 2: Try with different domain variations
        let domain_variations = [
            url.replace("www.facebook.com", "facebook.com"),
            url.replace("www.facebook.com", "web.facebook.com"),
            url.replace("facebook.com", "fb.com"),
        ];

        for variant_url in &domain_variations {
            if variant_url != url {
                info!("Trying domain variation: {}", variant_url);
                match self.try_direct_fetch(variant_url).await {
                    Ok(info) => {
                        info!("Domain variation extraction succeeded");
                        return Ok(info);
                    }
                    Err(_) => continue,
                }
            }
        }

        Err(FacebookExtractorError::html_parsing(
            "All alternative public extraction methods failed"
        ))
    }

    /// Try reel-specific extraction
    async fn try_reel_extraction(&self, url: &str) -> Result<VideoInfo> {
        info!("Trying reel-specific extraction");

        // Use mobile version for reels as they work better
        let mobile_url = url.replace("www.facebook.com", "m.facebook.com");

        let response = self
            .client
            .get(&mobile_url)
            .header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(FacebookExtractorError::html_parsing(format!(
                "Reel fetch failed with status: {}",
                response.status()
            )));
        }

        let html = response.text().await?;
        self.parse_facebook_reel_html(&html, url).await
    }

    /// Decode response bytes handling compression


    /// Clean view-source content for parsing
    fn clean_view_source_content(&self, content: &str) -> Result<String> {
        // Remove view-source: prefix if present
        let cleaned = content.strip_prefix("view-source:").unwrap_or(content);

        // Decode HTML entities
        let decoded = self.url_cleaner.decode_html_entities(cleaned);

        // Fix escaped JSON
        let fixed_json = self.url_cleaner.fix_escaped_json(&decoded);

        Ok(fixed_json)
    }

    /// Extract video ID from HTML content
    fn extract_video_id_from_html(&self, html: &str) -> Option<String> {
        let patterns = [
            r#""video_id":"(\d+)""#,
            r#""videoID":"(\d+)""#,
            r#""id":"(\d+)""#,
            r#"video_id=(\d+)"#,
        ];

        for pattern in &patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(id_match) = capture.get(1) {
                        return Some(id_match.as_str().to_string());
                    }
                }
            }
        }

        None
    }

    /// Parse Facebook HTML for video URLs (migrated from CLI implementation)
    async fn parse_facebook_html(&self, html: &str, original_url: &str) -> Result<VideoInfo> {
        info!("Parsing HTML for video URLs");

        // Check for authentication/blocking issues but be less aggressive
        let auth_indicators = html.contains("login") || html.contains("Log In");
        let captcha_detected = html.contains("captcha") || html.contains("CAPTCHA");

        if auth_indicators {
            warn!("Authentication indicators detected, but attempting extraction anyway for public content");
        }

        if captcha_detected {
            warn!("CAPTCHA detected - Facebook is blocking automated access, but attempting extraction anyway");
        }

        // Check for strong privacy indicators that definitely require auth
        let strong_privacy_indicators = [
            "This content isn't available right now",
            "This video is private",
            "Only friends can see this",
            "You must log in to continue",
            "Sign up for Facebook to continue",
            "Create an account or log in to Facebook"
        ];

        let has_strong_privacy = strong_privacy_indicators.iter()
            .any(|indicator| html.contains(indicator));

        if has_strong_privacy {
            warn!("Strong privacy indicators detected - video likely requires authentication");
            // Don't immediately fail - try extraction first, fail only if no video URLs found
        }

        if html.len() < 1000 {
            warn!("Response too short - possible blocking");
            return Err(FacebookExtractorError::html_parsing(
                "Response too short - Facebook may be blocking the request"
            ));
        }

        let video_id = self.extract_video_id(original_url)
            .or_else(|_| self.extract_video_id_from_html(html).ok_or_else(|| {
                FacebookExtractorError::video_id_extraction("Could not extract video ID")
            }))?;

        // Debug: Show what we're working with
        debug!("HTML Analysis:");
        debug!("  - HTML length: {} characters", html.len());
        debug!("  - Contains 'fbcdn.net': {}", html.contains("fbcdn.net"));
        debug!("  - Contains '.mp4': {}", html.contains(".mp4"));
        debug!("  - Contains 'video': {}", html.contains("video"));
        debug!("  - Contains 'playable_url': {}", html.contains("playable_url"));
        debug!("  - Contains 'browser_native': {}", html.contains("browser_native"));
        debug!("  - Contains 'dash_manifest': {}", html.contains("dash_manifest"));

        // Enhanced video URL patterns with better parameter capture
        let video_url_patterns = vec![
            // Direct video URLs
            r#"https://video[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
            r#"https://scontent[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
            r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
            // JSON embedded URLs
            r#""playable_url":"([^"]*\.mp4[^"]*)"#,
            r#""playable_url_quality_hd":"([^"]*\.mp4[^"]*)"#,
            r#""browser_native_hd_url":"([^"]*\.mp4[^"]*)"#,
            r#""browser_native_sd_url":"([^"]*\.mp4[^"]*)"#,
            r#""src":"([^"]*\.mp4[^"]*)"#,
            r#""url":"([^"]*\.mp4[^"]*)"#,
            // CRITICAL: base_url pattern - this is where Facebook stores multiple video qualities!
            r#""base_url":"([^"]*\.mp4[^"]*)"#,
            r#""base_url":"([^"]*)"#,  // Also capture non-.mp4 base URLs
            // Alternative formats
            r#""dash_manifest":"([^"]*)"#,
            r#""hls_playlist":"([^"]*)"#,
            // Escaped URLs
            r#"https:\\\/\\\/[^"]*\.fbcdn\.net[^"]*\.mp4[^"\s]*"#,
        ];

        let mut found_urls = Vec::new();

        for (i, pattern) in video_url_patterns.iter().enumerate() {
            debug!("Trying pattern {}/{}: {}", i + 1, video_url_patterns.len(), pattern);

            if let Ok(regex) = Regex::new(pattern) {
                let matches: Vec<_> = regex.captures_iter(html).collect();
                debug!("Found {} matches", matches.len());

                for capture in matches {
                    if let Some(url_match) = capture.get(1).or_else(|| capture.get(0)) {
                        let mut url = url_match.as_str().replace("\\", "");

                        // Clean up escaped characters
                        url = url.replace("\\/", "/");
                        url = url.replace("\\u0026", "&");

                        if (url.contains(".mp4") || url.contains("dash") || url.contains("m3u8"))
                            && !found_urls.contains(&url)
                        {
                            debug!("Found video URL: {}", &url[..80.min(url.len())]);
                            found_urls.push(url);
                        }
                    }
                }
            }
        }

        // Enhanced fallback: Search for audio streams specifically
        debug!("Searching for audio streams with EFG metadata");

        let audio_patterns = vec![
            r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*efg=[^"]*dash_ln_heaac[^"\s]*"#,
            r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*efg=[^"]*_audio[^"\s]*"#,
            r#"https:\\\/\\\/[^"]*\.fbcdn\.net[^"]*\.mp4[^"\s]*efg=[^"]*dash_ln_heaac[^"\s]*"#,
        ];

        for (i, pattern) in audio_patterns.iter().enumerate() {
            debug!("Trying audio pattern {}/{}", i + 1, audio_patterns.len());

            if let Ok(regex) = Regex::new(pattern) {
                let matches: Vec<_> = regex.find_iter(html).collect();
                debug!("Found {} audio stream matches", matches.len());

                for m in matches {
                    let mut url = m.as_str().replace("\\", "");
                    url = url.replace("\\/", "/");
                    url = url.replace("\\u0026", "&");

                    if !found_urls.contains(&url) {
                        debug!("Found audio stream URL: {}", &url[..100.min(url.len())]);
                        found_urls.push(url);
                    }
                }
            }
        }

        if found_urls.is_empty() {
            error!("No video URLs found in HTML response");
            debug!("HTML sample (first 1000 chars): {}", &html[..1000.min(html.len())]);

            // Provide more specific error messages based on what we detected
            let error_message = if has_strong_privacy {
                "No video URLs found - Video appears to be private and requires Facebook authentication"
            } else if captcha_detected {
                "No video URLs found - Facebook CAPTCHA is blocking access. Try using a different approach or wait before retrying."
            } else if auth_indicators {
                "No video URLs found - Facebook is requesting login, but this may be a public video. Try again or check if the video is publicly accessible."
            } else {
                "No video URLs found in HTML response - Facebook may have changed their format or the video may not be publicly accessible"
            };

            return Err(FacebookExtractorError::html_parsing(error_message));
        }

        info!("Found {} video URLs", found_urls.len());
        for (i, url) in found_urls.iter().enumerate() {
            debug!("Video URL {}: {}", i + 1, &url[..100.min(url.len())]);
        }

        // Store URLs for potential fallback use
        let urls_backup = found_urls.clone();

        // Extract comprehensive metadata first to get actual duration for accurate file size estimation
        let title = self.metadata_extractor.extract_title_from_html(html, &video_id);
        let metadata = self.metadata_extractor.extract_video_metadata(html);
        let duration = self.metadata_extractor.extract_duration_from_html(html);

        // Analyze and filter video streams with actual duration for accurate file size estimation
        // Analyze streams with accurate file size detection
        let mut all_qualities = Vec::new();
        for (index, url) in found_urls.into_iter().enumerate() {
            debug!("Processing URL {}: {}", index + 1, &url[..80.min(url.len())]);
            match self.stream_analyzer.analyze_facebook_video_stream_with_accurate_size(&url, metadata.duration_seconds).await {
                quality if quality.download_url.is_empty() => {
                    warn!("Failed to analyze stream {}: empty URL", index + 1);
                }
                quality => {
                    debug!("Successfully analyzed stream {}: {}p - {} MB", index + 1, quality.height, quality.estimated_size_mb);
                    all_qualities.push(quality);
                }
            }
        }

        if all_qualities.is_empty() {
            error!("No valid video streams could be analyzed");
            return Err(FacebookExtractorError::stream_analysis(
                "Failed to analyze any video streams - URLs may be invalid or incomplete"
            ));
        }

        // Filter for complete video+audio streams
        let mut complete_streams = self.stream_analyzer.filter_complete_video_streams(all_qualities);

        info!("Stream Analysis Complete: {} complete streams", complete_streams.len());

        if complete_streams.is_empty() {
            warn!("No complete video+audio streams found, using available streams");
            // Use all available streams as fallback with actual duration for size estimation
            let fallback_duration = metadata.duration_seconds.unwrap_or(120);
            complete_streams = urls_backup
                .into_iter()
                .enumerate()
                .filter_map(|(index, url)| {
                    if url.contains(".mp4") {
                        debug!("Creating fallback stream {}: {}", index + 1, &url[..80.min(url.len())]);
                        let is_hd = url.contains("hd");
                        let (width, height) = if is_hd { (1280, 720) } else { (854, 480) };

                        // Calculate realistic file size based on actual duration
                        let video_bitrate = if is_hd { 1500 } else { 800 }; // kbps
                        let audio_bitrate = 128; // kbps
                        let total_bitrate = video_bitrate + audio_bitrate;
                        let estimated_size_mb = (total_bitrate * fallback_duration) / 8 / 1024;

                        Some(VideoQuality {
                            quality: if is_hd { "HD".to_string() } else { "SD".to_string() },
                            size: format!("~{} MB", estimated_size_mb),
                            format: "mp4".to_string(),
                            download_url: url.clone(),
                            width,
                            height,
                            stream_type: StreamType::CompleteVideoAudio,
                            efg_metadata: "fallback".to_string(),
                            estimated_size_mb,
                            bitrate_kbps: Some(total_bitrate),
                            fps: Some(30),
                            codec: Some("h264".to_string()),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            if complete_streams.is_empty() {
                return Err(FacebookExtractorError::stream_analysis(
                    "No usable video streams found - all URLs appear to be invalid"
                ));
            }

            info!("Using {} fallback streams", complete_streams.len());
        }

        // Detect video aspect ratio from streams
        let video_aspect_ratio = if let Some(best_stream) = complete_streams.first() {
            AspectRatio::from_dimensions(best_stream.width, best_stream.height)
        } else {
            AspectRatio::Landscape16x9
        };

        // Extract enhanced thumbnail collection
        let thumbnail_collection = self.metadata_extractor
            .extract_thumbnail_collection(html, video_aspect_ratio)
            .await;

        // Legacy thumbnail for backward compatibility
        let thumbnail = thumbnail_collection.default_thumbnail.clone();

        // Determine privacy level from metadata or HTML
        let privacy_level = metadata.privacy_level.clone().unwrap_or_else(|| {
            // Fallback: try to detect from HTML content
            self.metadata_extractor.detect_privacy_level_from_html(html)
                .unwrap_or(PrivacyLevel::Unknown)
        });

        let video_info = VideoInfo {
            title,
            duration,
            thumbnail,
            thumbnail_variants: thumbnail_collection,
            qualities: complete_streams,
            video_id,
            metadata,
            extraction_timestamp: chrono::Utc::now(),
            source_url: original_url.to_string(),
            content_type: FacebookContentType::RegularVideo,
            privacy_level,
            access_method: AccessMethod::Direct,
        };

        Ok(video_info)
    }

    /// Parse Facebook HTML specifically for reels with mobile-optimized extraction
    async fn parse_facebook_reel_html(&self, html: &str, original_url: &str) -> Result<VideoInfo> {
        info!("Parsing HTML for reel content with mobile-optimized extraction");

        if html.len() < 1000 {
            warn!("Response too short - possible blocking");
            return Err(FacebookExtractorError::html_parsing(
                "Response too short - Facebook may be blocking the request"
            ));
        }

        let video_id = self.extract_video_id(original_url)
            .or_else(|_| self.extract_video_id_from_html(html).ok_or_else(|| {
                FacebookExtractorError::video_id_extraction("Could not extract video ID")
            }))?;

        // Use the same video URL extraction as regular videos
        // (This part works fine, the issue is with metadata extraction)
        let video_url_patterns = vec![
            r#"https://video[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
            r#"https://scontent[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
            r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
            r#""playable_url":"([^"]*\.mp4[^"]*)"#,
            r#""base_url":"([^"]*\.mp4[^"]*)"#,
            r#""base_url":"([^"]*)"#,
        ];

        let mut found_urls = Vec::new();

        for pattern in video_url_patterns.iter() {
            if let Ok(regex) = Regex::new(pattern) {
                let matches: Vec<_> = regex.captures_iter(html).collect();
                for capture in matches {
                    if let Some(url_match) = capture.get(1).or_else(|| capture.get(0)) {
                        let mut url = url_match.as_str().replace("\\", "");
                        url = url.replace("\\/", "/");
                        url = url.replace("\\u0026", "&");

                        if (url.contains(".mp4") || url.contains("dash") || url.contains("m3u8"))
                            && !found_urls.contains(&url)
                        {
                            found_urls.push(url);
                        }
                    }
                }
            }
        }

        if found_urls.is_empty() {
            return Err(FacebookExtractorError::html_parsing(
                "No video URLs found in reel content"
            ));
        }

        // REEL-SPECIFIC METADATA EXTRACTION FIRST
        // Extract metadata first to get actual duration for accurate file size estimation
        let title = self.extract_reel_title_from_html(html, &video_id);
        let author = self.extract_reel_author_from_html(html);
        let mut metadata = self.metadata_extractor.extract_video_metadata(html);

        // Override author with reel-specific extraction if found
        if !author.is_empty() && author != "Unknown Author" {
            metadata.author = author;
        }

        let duration = self.metadata_extractor.extract_duration_from_html(html);

        // Analyze and filter video streams with accurate file size estimation
        let mut all_qualities = Vec::new();
        for (index, url) in found_urls.into_iter().enumerate() {
            match self.stream_analyzer.analyze_facebook_video_stream_with_accurate_size(&url, metadata.duration_seconds).await {
                quality if quality.download_url.is_empty() => {
                    debug!("Skipping empty URL for stream {}", index + 1);
                }
                quality => {
                    debug!("Analyzed reel stream {}: {} MB", index + 1, quality.estimated_size_mb);
                    all_qualities.push(quality);
                }
            }
        }

        let complete_streams = if all_qualities.is_empty() {
            return Err(FacebookExtractorError::stream_analysis(
                "Failed to analyze any video streams"
            ));
        } else {
            self.stream_analyzer.filter_complete_video_streams(all_qualities)
        };

        // Detect video aspect ratio from streams
        let video_aspect_ratio = if let Some(best_stream) = complete_streams.first() {
            AspectRatio::from_dimensions(best_stream.width, best_stream.height)
        } else {
            AspectRatio::Portrait9x16  // Reels are typically portrait
        };

        // Extract enhanced thumbnail collection
        let thumbnail_collection = self.metadata_extractor
            .extract_thumbnail_collection(html, video_aspect_ratio)
            .await;

        let thumbnail = thumbnail_collection.default_thumbnail.clone();

        // Determine privacy level from metadata or HTML
        let privacy_level = metadata.privacy_level.clone().unwrap_or_else(|| {
            // Fallback: try to detect from HTML content
            self.metadata_extractor.detect_privacy_level_from_html(html)
                .unwrap_or(PrivacyLevel::Unknown)
        });

        Ok(VideoInfo {
            title,
            duration,
            thumbnail,
            thumbnail_variants: thumbnail_collection,
            qualities: complete_streams,
            video_id,
            metadata,
            extraction_timestamp: chrono::Utc::now(),
            source_url: original_url.to_string(),
            content_type: FacebookContentType::Reel,
            privacy_level,
            access_method: AccessMethod::Mobile,
        })
    }

    /// Extract title specifically for reels using mobile format patterns
    fn extract_reel_title_from_html(&self, html: &str, video_id: &str) -> String {
        // First try the mobile title format: "Title | Author | Facebook"
        if let Ok(regex) = Regex::new(r#"<title>([^<]+)</title>"#) {
            if let Some(capture) = regex.captures(html) {
                if let Some(title_match) = capture.get(1) {
                    let raw_title = title_match.as_str();

                    // Decode HTML entities and Unicode escapes
                    let decoded_title = self.metadata_extractor.decode_html_entities(raw_title);
                    let decoded_title = self.metadata_extractor.decode_unicode_escapes(&decoded_title);

                    // Handle mobile Facebook format: "Title | Author | Facebook"
                    if decoded_title.contains(" | ") {
                        let parts: Vec<&str> = decoded_title.split(" | ").collect();

                        if parts.len() >= 2 {
                            let potential_title = parts[0].trim();

                            // Skip if this looks like engagement data or technical content
                            if !self.metadata_extractor.is_engagement_data(potential_title) &&
                               !self.metadata_extractor.is_technical_content(potential_title) &&
                               potential_title.len() > 5 {
                                return potential_title.to_string();
                            }
                        }
                    }
                }
            }
        }

        // Fallback to regular title extraction
        self.metadata_extractor.extract_title_from_html(html, video_id)
    }

    /// Extract author specifically for reels using mobile format patterns
    fn extract_reel_author_from_html(&self, html: &str) -> String {
        // First try the mobile title format: "Title | Author | Facebook"
        if let Ok(regex) = Regex::new(r#"<title>([^<]+)</title>"#) {
            if let Some(capture) = regex.captures(html) {
                if let Some(title_match) = capture.get(1) {
                    let raw_title = title_match.as_str();

                    // Decode HTML entities and Unicode escapes
                    let decoded_title = self.metadata_extractor.decode_html_entities(raw_title);
                    let decoded_title = self.metadata_extractor.decode_unicode_escapes(&decoded_title);

                    // Handle mobile Facebook format: "Title | Author | Facebook"
                    if decoded_title.contains(" | ") {
                        let parts: Vec<&str> = decoded_title.split(" | ").collect();

                        if parts.len() >= 2 {
                            let potential_author = parts[1].trim()
                                .replace(" - Facebook", "")
                                .replace(" | Facebook", "")
                                .trim()
                                .to_string();

                            // Check if this looks like an author name
                            if !potential_author.is_empty() &&
                               potential_author != "Facebook" &&
                               potential_author.len() > 2 &&
                               !self.metadata_extractor.is_engagement_data(&potential_author) {
                                return potential_author;
                            }
                        }
                    }
                }
            }
        }

        // Fallback to regular author extraction
        match self.metadata_extractor.extract_author_from_title(html) {
            Some(author) => author,
            None => self.metadata_extractor.extract_author_from_html(html)
        }
    }

    /// Parse Facebook HTML bypassing all authentication checks (for public videos)
    async fn parse_facebook_html_bypass_auth(&self, html: &str, original_url: &str) -> Result<VideoInfo> {
        info!("Parsing HTML with authentication bypass for public content");

        // Skip all authentication checks and go straight to extraction
        if html.len() < 500 {
            warn!("Response very short - may not contain video data");
            return Err(FacebookExtractorError::html_parsing(
                "Response too short to contain video data"
            ));
        }

        let video_id = self.extract_video_id(original_url)
            .or_else(|_| self.extract_video_id_from_html(html).ok_or_else(|| {
                FacebookExtractorError::video_id_extraction("Could not extract video ID")
            }))?;

        // Use more aggressive video URL patterns for public content
        let public_video_patterns = vec![
            // Direct CDN URLs (most reliable for public videos)
            r#"https://video[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
            r#"https://scontent[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
            r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
            // JSON embedded URLs with more flexible matching
            r#""[^"]*_url":"([^"]*\.mp4[^"]*)"#,
            r#""playable_url[^"]*":"([^"]*\.mp4[^"]*)"#,
            r#""browser_native[^"]*":"([^"]*\.mp4[^"]*)"#,
            r#""src":"([^"]*\.mp4[^"]*)"#,
            r#""url":"([^"]*\.mp4[^"]*)"#,
            r#""base_url":"([^"]*\.mp4[^"]*)"#,
            // Alternative formats
            r#""dash_manifest":"([^"]*)"#,
            r#""hls_playlist":"([^"]*)"#,
            // Escaped URLs
            r#"https:\\\/\\\/[^"]*\.fbcdn\.net[^"]*\.mp4[^"\s]*"#,
            // More permissive patterns for public content
            r#"(https://[^"]*\.mp4[^"\s]*)"#,
        ];

        let mut found_urls = Vec::new();

        for (i, pattern) in public_video_patterns.iter().enumerate() {
            debug!("Trying public pattern {}/{}: {}", i + 1, public_video_patterns.len(), pattern);

            if let Ok(regex) = Regex::new(pattern) {
                let matches: Vec<_> = regex.captures_iter(html).collect();
                debug!("Found {} matches", matches.len());

                for capture in matches {
                    if let Some(url_match) = capture.get(1).or_else(|| capture.get(0)) {
                        let mut url = url_match.as_str().replace("\\", "");

                        // Clean up escaped characters
                        url = url.replace("\\/", "/");
                        url = url.replace("\\u0026", "&");

                        if (url.contains(".mp4") || url.contains("dash") || url.contains("m3u8"))
                            && !found_urls.contains(&url)
                        {
                            debug!("Found public video URL: {}", &url[..80.min(url.len())]);
                            found_urls.push(url);
                        }
                    }
                }
            }
        }

        if found_urls.is_empty() {
            error!("No video URLs found in public content");
            return Err(FacebookExtractorError::html_parsing(
                "No video URLs found - content may not be publicly accessible or Facebook has changed their format"
            ));
        }

        info!("Found {} video URLs in public content", found_urls.len());

        // Create basic video qualities from found URLs
        let qualities: Vec<VideoQuality> = found_urls
            .into_iter()
            .enumerate()
            .map(|(index, url)| {
                let quality = if url.contains("hd") || url.contains("720") || url.contains("1080") {
                    "HD".to_string()
                } else {
                    "SD".to_string()
                };

                VideoQuality {
                    quality: quality.clone(),
                    size: "Unknown".to_string(),
                    format: "mp4".to_string(),
                    download_url: url.clone(),
                    width: if quality == "HD" { 1280 } else { 854 },
                    height: if quality == "HD" { 720 } else { 480 },
                    stream_type: StreamType::CompleteVideoAudio,
                    efg_metadata: format!("public_bypass_{}", index),
                    estimated_size_mb: if quality == "HD" { 50 } else { 25 },
                    bitrate_kbps: Some(if quality == "HD" { 2500 } else { 1000 }),
                    fps: Some(30),
                    codec: Some("h264".to_string()),
                }
            })
            .collect();

        // Extract basic metadata
        let title = self.metadata_extractor.extract_title_from_html(html, &video_id);
        let metadata = self.metadata_extractor.extract_video_metadata(html);
        let duration = self.metadata_extractor.extract_duration_from_html(html);

        // Detect video aspect ratio from streams
        let video_aspect_ratio = if let Some(best_stream) = qualities.first() {
            AspectRatio::from_dimensions(best_stream.width, best_stream.height)
        } else {
            AspectRatio::Landscape16x9
        };

        // Extract enhanced thumbnail collection
        let thumbnail_collection = self.metadata_extractor
            .extract_thumbnail_collection(html, video_aspect_ratio)
            .await;

        // Legacy thumbnail for backward compatibility
        let thumbnail = thumbnail_collection.default_thumbnail.clone();

        Ok(VideoInfo {
            title,
            duration,
            thumbnail,
            thumbnail_variants: thumbnail_collection,
            qualities,
            video_id,
            metadata,
            extraction_timestamp: chrono::Utc::now(),
            source_url: original_url.to_string(),
            content_type: FacebookContentType::RegularVideo,
            privacy_level: PrivacyLevel::Public,
            access_method: AccessMethod::Alternative,
        })
    }
}
