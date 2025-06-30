//! # Common Test Utilities for Facebook Video Extraction
//!
//! This module provides shared utilities, data structures, and helper functions
//! used across all Facebook video extraction tests to eliminate code duplication
//! and ensure consistency.

use reqwest::Client;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use chrono;

// ============================================================================
// SHARED ERROR TYPES
// ============================================================================

/// Common error types for extraction tests
#[derive(Debug)]
pub enum ExtractionTestError {
    InvalidUrl(String),
    VideoIdExtraction(String),
    NetworkError(reqwest::Error),
    HtmlParsingError(String),
    AuthenticationRequired(String),
    PrivateVideoError(String),
    RateLimited(String),
    ContentUnavailable(String),
    StreamAnalysisError(String),
    TimeoutError(String),
    ConfigError(String),
}

impl std::fmt::Display for ExtractionTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractionTestError::InvalidUrl(url) => write!(f, "Invalid Facebook URL: {}", url),
            ExtractionTestError::VideoIdExtraction(msg) => write!(f, "Video ID extraction failed: {}", msg),
            ExtractionTestError::NetworkError(err) => write!(f, "Network request failed: {}", err),
            ExtractionTestError::HtmlParsingError(msg) => write!(f, "HTML parsing failed: {}", msg),
            ExtractionTestError::AuthenticationRequired(msg) => write!(f, "Authentication required: {}", msg),
            ExtractionTestError::PrivateVideoError(msg) => write!(f, "Private video access denied: {}", msg),
            ExtractionTestError::RateLimited(msg) => write!(f, "Rate limit exceeded: {}", msg),
            ExtractionTestError::ContentUnavailable(msg) => write!(f, "Content unavailable: {}", msg),
            ExtractionTestError::StreamAnalysisError(msg) => write!(f, "Stream analysis failed: {}", msg),
            ExtractionTestError::TimeoutError(msg) => write!(f, "Timeout occurred: {}", msg),
            ExtractionTestError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for ExtractionTestError {}

impl From<reqwest::Error> for ExtractionTestError {
    fn from(err: reqwest::Error) -> Self {
        ExtractionTestError::NetworkError(err)
    }
}

impl From<std::io::Error> for ExtractionTestError {
    fn from(err: std::io::Error) -> Self {
        ExtractionTestError::ConfigError(format!("IO error: {}", err))
    }
}

pub type TestResult<T> = std::result::Result<T, ExtractionTestError>;

// ============================================================================
// SHARED DATA STRUCTURES
// ============================================================================

/// Stream type classification for Facebook videos
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamType {
    CompleteVideoAudio,
    VideoOnly,
    AudioOnly,
    CombinedVideoAudio,
    Unknown,
}

impl StreamType {
    pub fn has_video(&self) -> bool {
        matches!(self, Self::CompleteVideoAudio | Self::VideoOnly | Self::CombinedVideoAudio)
    }

    pub fn has_audio(&self) -> bool {
        matches!(self, Self::CompleteVideoAudio | Self::AudioOnly | Self::CombinedVideoAudio)
    }

    pub fn is_complete(&self) -> bool {
        matches!(self, Self::CompleteVideoAudio | Self::CombinedVideoAudio)
    }
}

/// Video quality information with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoQuality {
    pub quality: String,
    pub size: String,
    pub format: String,
    pub download_url: String,
    pub width: u32,
    pub height: u32,
    pub stream_type: StreamType,
    pub estimated_size_mb: u32,
    pub bitrate_kbps: Option<u32>,
    pub fps: Option<u32>,
    pub codec: Option<String>,
}

impl VideoQuality {
    pub fn pixel_count(&self) -> u32 {
        self.width * self.height
    }

    pub fn quality_score(&self) -> u32 {
        let pixel_score = self.pixel_count();
        let bitrate_score = self.bitrate_kbps.unwrap_or(0);
        let complete_bonus = if self.stream_type.is_complete() { 1000000 } else { 0 };
        pixel_score + bitrate_score + complete_bonus
    }
}

/// Enhanced video metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub author: String,
    pub description: String,
    pub publish_date: String,
    pub likes: u64,
    pub comments: u64,
    pub views: u64,
    pub shares: u64,
    pub hashtags: Vec<String>,
    pub duration_seconds: Option<u32>,
    pub language: Option<String>,
    pub category: Option<String>,
}

impl Default for VideoMetadata {
    fn default() -> Self {
        Self {
            author: "Unknown Author".to_string(),
            description: "No description available".to_string(),
            publish_date: "Unknown date".to_string(),
            likes: 0,
            comments: 0,
            views: 0,
            shares: 0,
            hashtags: Vec::new(),
            duration_seconds: None,
            language: None,
            category: None,
        }
    }
}

/// Complete video information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub title: String,
    pub duration: String,
    pub thumbnail: String,
    pub qualities: Vec<VideoQuality>,
    pub video_id: String,
    pub metadata: VideoMetadata,
    pub extraction_timestamp: chrono::DateTime<chrono::Utc>,
    pub source_url: String,
    pub privacy_status: String,
    pub access_method: String,
}

impl VideoInfo {
    pub fn new(title: String, video_id: String, source_url: String) -> Self {
        Self {
            title,
            duration: "Unknown duration".to_string(),
            thumbnail: String::new(),
            qualities: Vec::new(),
            video_id,
            metadata: VideoMetadata::default(),
            extraction_timestamp: chrono::Utc::now(),
            source_url,
            privacy_status: "Unknown".to_string(),
            access_method: "Direct".to_string(),
        }
    }

    pub fn best_quality(&self) -> Option<&VideoQuality> {
        self.qualities.iter().max_by_key(|q| q.quality_score())
    }

    pub fn complete_streams(&self) -> Vec<&VideoQuality> {
        self.qualities.iter().filter(|q| q.stream_type.is_complete()).collect()
    }
}

// ============================================================================
// SHARED HTTP CLIENT CONFIGURATION
// ============================================================================

/// HTTP client configuration for consistent behavior across tests
#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub timeout: Duration,
    pub connection_timeout: Duration,
    pub max_redirects: usize,
    pub user_agents: Vec<String>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            max_redirects: 5,
            user_agents: vec![
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
                "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Mobile/15E148 Safari/604.1".to_string(),
            ],
        }
    }
}

/// Create a configured HTTP client for testing
pub fn create_test_client(config: Option<HttpConfig>) -> TestResult<Client> {
    let config = config.unwrap_or_default();
    
    Client::builder()
        .timeout(config.timeout)
        .connect_timeout(config.connection_timeout)
        .redirect(reqwest::redirect::Policy::limited(config.max_redirects))
        .build()
        .map_err(|e| ExtractionTestError::ConfigError(format!("Failed to create HTTP client: {}", e)))
}

/// Create a mobile-optimized HTTP client
pub fn create_mobile_client() -> TestResult<Client> {
    let config = HttpConfig {
        user_agents: vec![
            "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Mobile/15E148 Safari/604.1".to_string()
        ],
        ..Default::default()
    };
    create_test_client(Some(config))
}

// ============================================================================
// SHARED URL VALIDATION AND EXTRACTION
// ============================================================================

/// Validate Facebook URL format using comprehensive regex patterns
pub fn is_valid_facebook_url(url: &str) -> bool {
    let patterns = [
        r"^https?://(?:www\.)?facebook\.com/watch\?.*v=\d+",
        r"^https?://(?:www\.)?facebook\.com/[^/]+/videos/\d+",
        r"^https?://fb\.watch/[a-zA-Z0-9_-]+",
        r"^https?://(?:www\.)?facebook\.com/video\.php\?v=\d+",
        r"^https?://(?:www\.)?facebook\.com/[^/]+/posts/\d+",
        r"^https?://(?:www\.)?facebook\.com/reel/\d+",
        r"^https?://(?:www\.)?facebook\.com/share/v/[a-zA-Z0-9_-]+/?",
        r"^https?://(?:www\.)?facebook\.com/share/r/[a-zA-Z0-9_-]+/?",
    ];

    patterns.iter().any(|pattern| {
        if let Ok(regex) = Regex::new(pattern) {
            regex.is_match(url)
        } else {
            false
        }
    })
}

/// Extract video ID from Facebook URL
pub fn extract_video_id(url: &str) -> TestResult<String> {
    let patterns = [
        r"facebook\.com/watch/?\?.*[&?]v=(\d+)",
        r"facebook\.com/watch/?\?v=(\d+)",
        r"facebook\.com/.*/videos/(\d+)",
        r"facebook\.com/reel/(\d+)",
        r"fb\.watch/([a-zA-Z0-9]+)",
        r"facebook\.com/share/v/([a-zA-Z0-9_-]+)",
        r"facebook\.com/share/r/([a-zA-Z0-9_-]+)",
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

    Err(ExtractionTestError::VideoIdExtraction(
        "Could not extract video ID from URL".to_string(),
    ))
}

// ============================================================================
// SHARED VIDEO URL PATTERNS
// ============================================================================

/// Get standard video URL extraction patterns
pub fn get_video_url_patterns() -> Vec<&'static str> {
    vec![
        r#"https://video[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
        r#"https://scontent[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
        r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
        r#""playable_url":"([^"]*\.mp4[^"]*)"#,
        r#""playable_url_quality_hd":"([^"]*\.mp4[^"]*)"#,
        r#""browser_native_hd_url":"([^"]*\.mp4[^"]*)"#,
        r#""browser_native_sd_url":"([^"]*\.mp4[^"]*)"#,
        r#""src":"([^"]*\.mp4[^"]*)"#,
        r#""url":"([^"]*\.mp4[^"]*)"#,
        r#""dash_manifest":"([^"]*)"#,
        r#""hls_playlist":"([^"]*)"#,
        r#"https:\\\/\\\/[^"]*\.fbcdn\.net[^"]*\.mp4[^"\s]*"#,
        r#"data-src="([^"]*\.mp4[^"]*)"#,
        r#"src="([^"]*\.mp4[^"]*)"#,
    ]
}

/// Get audio stream specific patterns
pub fn get_audio_stream_patterns() -> Vec<&'static str> {
    vec![
        r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*efg=[^"]*dash_ln_heaac[^"\s]*"#,
        r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*efg=[^"]*_audio[^"\s]*"#,
        r#"https:\\\/\\\/[^"]*\.fbcdn\.net[^"]*\.mp4[^"\s]*efg=[^"]*dash_ln_heaac[^"\s]*"#,
        r#"https:\\\/\\\/[^"]*\.fbcdn\.net[^"]*\.mp4[^"\s]*efg=[^"]*_audio[^"\s]*"#,
        r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"dash_ln_heaac_vbr3_audio"[^"\s]*"#,
        r#"https:\\\/\\\/[^"]*\.fbcdn\.net[^"]*\.mp4[^"\s]*"dash_ln_heaac_vbr3_audio"[^"\s]*"#,
    ]
}

// ============================================================================
// SHARED HTML ANALYSIS FUNCTIONS
// ============================================================================

/// Extract URLs from HTML using provided patterns
pub fn extract_urls_from_html(html: &str, patterns: &[&str]) -> Vec<String> {
    let mut found_urls = Vec::new();

    for pattern in patterns {
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

    found_urls
}

/// Check for authentication/blocking indicators in HTML
pub fn check_authentication_status(html: &str) -> (bool, bool, bool) {
    let auth_indicators = [
        "login", "Log In", "log in", "sign in", "Sign In",
        "authentication", "private", "Private", "friends only",
        "not available", "content not found", "video not found",
        "requires login", "sign up", "create account",
        "loginform", "login_form", "signin", "auth",
    ];

    let blocking_indicators = [
        "captcha", "CAPTCHA", "security check", "unusual activity",
        "automated", "bot", "suspicious", "verify", "challenge",
        "rate limit", "too many requests", "blocked",
    ];

    let privacy_indicators = [
        "This content isn't available right now",
        "This video is private",
        "Only friends can see this",
        "This post is no longer available",
        "Content not found",
        "Video unavailable",
    ];

    let auth_detected = auth_indicators.iter()
        .any(|indicator| html.to_lowercase().contains(&indicator.to_lowercase()));

    let blocking_detected = blocking_indicators.iter()
        .any(|indicator| html.to_lowercase().contains(&indicator.to_lowercase()));

    let privacy_detected = privacy_indicators.iter()
        .any(|indicator| html.contains(indicator));

    (auth_detected, blocking_detected, privacy_detected)
}

// ============================================================================
// SHARED TEST DATA AND UTILITIES
// ============================================================================

/// Common test URLs for consistent testing across all extraction tests
pub struct TestUrls;

impl TestUrls {
    /// Get a set of known working public Facebook video URLs for testing
    pub fn get_working_public_urls() -> Vec<&'static str> {
        vec![
            "https://www.facebook.com/watch/?v=2209933269449948",
            "https://www.facebook.com/watch?v=419280024562892",
            "https://www.facebook.com/watch/?v=1193939392365151",
            "https://www.facebook.com/watch/?v=900186485344685",
            "https://www.facebook.com/watch?v=999173769094576",
        ]
    }

    /// Get URLs for testing different Facebook URL formats
    pub fn get_format_test_urls() -> Vec<&'static str> {
        vec![
            "https://www.facebook.com/watch/?v=2209933269449948",
            "https://www.facebook.com/reel/1193939392365151",
            "https://m.facebook.com/watch/?v=2209933269449948",
            "https://fb.watch/abc123def",
        ]
    }

    /// Get URLs that are likely to be private or require authentication
    pub fn get_private_test_urls() -> Vec<&'static str> {
        vec![
            "https://www.facebook.com/watch/?v=123456789",
            "https://www.facebook.com/private-user/videos/123456789",
        ]
    }
}

/// Test configuration for extraction tests
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub save_debug_files: bool,
    pub verbose_logging: bool,
    pub timeout_seconds: u64,
    pub max_retries: usize,
    pub test_private_videos: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            save_debug_files: false,
            verbose_logging: false,
            timeout_seconds: 30,
            max_retries: 3,
            test_private_videos: false,
        }
    }
}

/// Save HTML content to debug file for inspection
pub fn save_debug_html(html: &str, filename: &str, config: &TestConfig) -> std::io::Result<()> {
    if config.save_debug_files {
        std::fs::write(filename, html)?;
        if config.verbose_logging {
            println!("💾 Saved debug HTML to: {}", filename);
        }
    }
    Ok(())
}

/// Print test progress with consistent formatting
pub fn print_test_progress(step: usize, total: usize, description: &str) {
    println!("🧪 Test {}/{}: {}", step, total, description);
    println!("   {}", "=".repeat(80));
}

/// Print test result summary
pub fn print_test_summary(passed: usize, failed: usize, skipped: usize) {
    println!("\n📊 TEST SUMMARY");
    println!("===============");
    println!("✅ Passed: {}", passed);
    println!("❌ Failed: {}", failed);
    println!("⏭️  Skipped: {}", skipped);
    println!("📋 Total: {}", passed + failed + skipped);

    let success_rate = if passed + failed > 0 {
        (passed as f64 / (passed + failed) as f64) * 100.0
    } else {
        0.0
    };
    println!("📈 Success Rate: {:.1}%", success_rate);
}

/// Convert mobile URL to desktop URL
pub fn mobile_to_desktop_url(url: &str) -> String {
    url.replace("m.facebook.com", "www.facebook.com")
}

/// Convert desktop URL to mobile URL
pub fn desktop_to_mobile_url(url: &str) -> String {
    url.replace("www.facebook.com", "m.facebook.com")
}

/// Analyze video quality from URL and metadata
pub fn analyze_video_stream(url: &str) -> VideoQuality {
    // Extract basic quality information from URL patterns
    let (width, height, quality_name) = if url.contains("hd") || url.contains("720") {
        (1280, 720, "HD 720p")
    } else if url.contains("sd") || url.contains("480") {
        (854, 480, "SD 480p")
    } else if url.contains("1080") {
        (1920, 1080, "Full HD 1080p")
    } else {
        (640, 360, "Standard Quality")
    };

    // Determine stream type based on URL patterns
    let stream_type = if url.contains("dash_ln_heaac") || url.contains("_audio") {
        StreamType::AudioOnly
    } else if url.contains("dash") && !url.contains("audio") {
        StreamType::VideoOnly
    } else {
        StreamType::CompleteVideoAudio
    };

    // Estimate file size based on quality and duration (rough estimate)
    let estimated_size_mb = match quality_name {
        "Full HD 1080p" => 100,
        "HD 720p" => 60,
        "SD 480p" => 30,
        _ => 20,
    };

    VideoQuality {
        quality: quality_name.to_string(),
        size: format!("{}x{}", width, height),
        format: "mp4".to_string(),
        download_url: url.to_string(),
        width,
        height,
        stream_type,
        estimated_size_mb,
        bitrate_kbps: None,
        fps: Some(30),
        codec: Some("H.264".to_string()),
    }
}

/// Extract title from HTML using common patterns
pub fn extract_title_from_html(html: &str, video_id: &str) -> String {
    let title_patterns = vec![
        r#"<title>([^<]+)</title>"#,
        r#""title":"([^"]+)""#,
        r#""name":"([^"]+)""#,
        r#"<meta property="og:title" content="([^"]+)""#,
    ];

    for pattern in title_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(capture) = regex.captures(html) {
                if let Some(title_match) = capture.get(1) {
                    let title = title_match.as_str().replace("\\", "");
                    if !title.is_empty() && title.len() > 5 {
                        return title;
                    }
                }
            }
        }
    }

    format!("Facebook Video {}", &video_id[..8.min(video_id.len())])
}
