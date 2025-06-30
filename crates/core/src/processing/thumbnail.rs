//! Enhanced thumbnail extraction and processing for Facebook videos

use crate::common::types::{ThumbnailVariant, ThumbnailCollection, AspectRatio, DisplayContext};
use regex::Regex;
use reqwest::Client;
use std::time::Duration;
use tracing::{info, warn};

/// Enhanced thumbnail extractor with support for multiple variants and aspect ratios
pub struct ThumbnailExtractor {
    client: Client,
}

impl ThumbnailExtractor {
    /// Create a new thumbnail extractor
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()?;
        
        Ok(Self { client })
    }
    
    /// Extract comprehensive thumbnail collection from HTML
    pub async fn extract_thumbnail_collection(
        &self,
        html: &str,
        video_aspect_ratio: AspectRatio,
    ) -> ThumbnailCollection {
        info!("🖼️ Starting enhanced thumbnail collection extraction");
        
        let mut collection = ThumbnailCollection::new(video_aspect_ratio);
        
        // Extract all potential thumbnail URLs from HTML
        let thumbnail_urls = self.extract_all_thumbnail_urls(html);
        info!("📊 Found {} potential thumbnail URLs", thumbnail_urls.len());
        
        // Process each URL and create variants
        for (index, url) in thumbnail_urls.iter().enumerate() {
            info!("🔄 Processing thumbnail URL {}/{}: {}", index + 1, thumbnail_urls.len(), &url[..100.min(url.len())]);
            
            match self.create_thumbnail_variants(url, video_aspect_ratio).await {
                Ok(variants) => {
                    for variant in variants {
                        collection.add_variant(variant);
                    }
                }
                Err(e) => {
                    warn!("⚠️ Failed to create variants for URL {}: {}", index + 1, e);
                }
            }
        }
        
        // If no variants were created, generate fallback variants
        if collection.variants.is_empty() {
            warn!("❌ No thumbnail variants created, generating fallbacks");
            self.generate_fallback_variants(&mut collection);
        }
        
        info!("✅ Thumbnail collection complete: {} variants", collection.variants.len());
        collection
    }
    
    /// Extract all potential thumbnail URLs from HTML
    fn extract_all_thumbnail_urls(&self, html: &str) -> Vec<String> {
        info!("🔍 Starting thumbnail URL extraction from HTML");

        // High-priority patterns for actual video thumbnails (try these first)
        let high_priority_patterns = vec![
            // CRITICAL: Facebook's preferred_thumbnail pattern (most important)
            r#""preferred_thumbnail":\{"image":\{"uri":"([^"]+)""#,

            // Video-specific thumbnail patterns
            r#""video_thumbnail":"([^"]+)""#,
            r#""video_preview_image":"([^"]+)""#,
            r#""playable_url_quality_hd_thumbnail":"([^"]+)""#,

            // Open Graph and meta tags (usually the main video thumbnail)
            r#"<meta property="og:image" content="([^"]+)""#,
            r#"<meta name="twitter:image" content="([^"]+)""#,
            r#"<meta property="og:image:url" content="([^"]+)""#,

            // Video poster and preview images
            r#""poster":"([^"]+)""#,
            r#""preview_image":"([^"]+)""#,
            r#""video_preview_image_url":"([^"]+)""#,
        ];

        // Try high-priority patterns first
        let high_priority_candidates = self.extract_urls_with_patterns(&high_priority_patterns, html);
        let filtered_high_priority = self.filter_video_thumbnails(high_priority_candidates);

        if !filtered_high_priority.is_empty() {
            info!("✅ Found {} high-priority thumbnail candidates", filtered_high_priority.len());
            return filtered_high_priority;
        }

        info!("⚠️ No high-priority thumbnails found, trying fallback patterns...");

        // Fallback patterns (only if high-priority fails)
        let fallback_patterns = vec![
            // Modern Facebook JSON-LD patterns
            r#""thumbnailUrl":"([^"]+)""#,
            r#""thumbnail":"([^"]+)""#,

            // Facebook-specific patterns
            r#""thumbnailImage":"([^"]+)""#,
            r#""previewImage":"([^"]+)""#,
            r#""image":\{"uri":"([^"]+)""#,
            r#""image_url":"([^"]+)""#,
            r#""cover_photo":\{"source":"([^"]+)""#,

            // Escaped JSON patterns
            r#"\\"thumbnail\\":\\"([^"]+)\\""#,
            r#"\\"image\\":\\"([^"]+)\\""#,

            // Alternative formats
            r#"thumbnail_url=([^&\s]+)"#,
            r#"preview_url=([^&\s]+)"#,
        ];

        let fallback_candidates = self.extract_urls_with_patterns(&fallback_patterns, html);
        let filtered_fallbacks = self.filter_video_thumbnails(fallback_candidates);

        if !filtered_fallbacks.is_empty() {
            info!("✅ Found {} fallback thumbnail candidates", filtered_fallbacks.len());
            return filtered_fallbacks;
        }

        warn!("❌ No thumbnail found in HTML after exhaustive search");
        Vec::new()
    }

    /// Extract URLs using given patterns
    fn extract_urls_with_patterns(&self, patterns: &[&str], html: &str) -> Vec<String> {
        let mut all_candidates = Vec::new();

        for (_index, pattern) in patterns.iter().enumerate() {
            if let Ok(regex) = Regex::new(pattern) {
                let matches: Vec<_> = regex.captures_iter(html).collect();

                for (_match_index, capture) in matches.iter().enumerate() {
                    if let Some(thumb_match) = capture.get(1) {
                        let mut thumbnail = thumb_match.as_str().to_string();

                        // Clean up escaped characters
                        thumbnail = thumbnail.replace("\\", "");
                        thumbnail = thumbnail.replace("\\/", "/");
                        thumbnail = thumbnail.replace("\\u0026", "&");

                        if thumbnail.starts_with("http") && thumbnail.len() > 20 {
                            all_candidates.push(thumbnail);
                        }
                    }
                }
            }
        }

        all_candidates
    }

    /// Filter URLs to only include likely video thumbnails
    fn filter_video_thumbnails(&self, candidates: Vec<String>) -> Vec<String> {
        let mut filtered = Vec::new();

        for url in candidates {
            // Skip obvious UI elements and small icons
            if self.is_likely_video_thumbnail(&url) {
                filtered.push(url);
            }
        }

        // Limit to top 5 candidates to avoid processing too many
        filtered.truncate(5);
        filtered
    }

    /// Check if URL is likely a video thumbnail (not a UI icon)
    fn is_likely_video_thumbnail(&self, url: &str) -> bool {
        // Skip obvious UI elements
        if url.contains("/rsrc.php/") ||
           url.contains("static.xx.fbcdn.net/rsrc.php/") ||
           url.contains("_16x16") ||
           url.contains("_24x24") ||
           url.contains("_32x32") ||
           url.contains("icon") ||
           url.contains("sprite") ||
           url.contains("emoji") {
            return false;
        }

        // Prefer video-related URLs
        if url.contains("video") ||
           url.contains("thumbnail") ||
           url.contains("preview") ||
           url.contains("scontent") {
            return true;
        }

        // Accept fbcdn.net URLs that look like media
        if url.contains("fbcdn.net") &&
           (url.contains(".jpg") || url.contains(".png") || url.contains(".webp")) &&
           !url.contains("static.xx.fbcdn.net") {
            return true;
        }

        false
    }

    /// Filter and prioritize thumbnails (legacy method for compatibility)
    fn filter_and_prioritize_thumbnails(&self, candidates: Vec<String>) -> Vec<(String, u32)> {
        candidates.into_iter()
            .filter(|url| self.is_likely_video_thumbnail(url))
            .map(|url| {
                let priority = if url.contains("video") || url.contains("thumbnail") {
                    100
                } else if url.contains("fbcdn.net") {
                    80
                } else {
                    50
                };
                (url, priority)
            })
            .collect()
    }
    
    /// Check if URL is a valid thumbnail URL
    fn is_valid_thumbnail_url(&self, url: &str) -> bool {
        // More permissive validation since we're already filtering in the patterns
        url.starts_with("http") && url.len() > 20
    }
    
    /// Create multiple thumbnail variants from a single URL
    async fn create_thumbnail_variants(
        &self,
        url: &str,
        video_aspect_ratio: AspectRatio,
    ) -> Result<Vec<ThumbnailVariant>, Box<dyn std::error::Error>> {
        // Download the original image
        let image_data = self.download_thumbnail_data(url).await?;
        let (original_width, original_height) = self.detect_image_dimensions(&image_data)?;
        
        info!("📐 Original thumbnail dimensions: {}x{}", original_width, original_height);
        
        let mut variants = Vec::new();
        
        // Create variants for different display contexts
        let contexts_and_sizes = vec![
            (DisplayContext::DesktopFullscreen, (1920, 1080)),
            (DisplayContext::MobilePortrait, (720, 1280)),
            (DisplayContext::MobileLandscape, (1280, 720)),
            (DisplayContext::WebThumbnail, (480, 270)),
            (DisplayContext::SocialShare, (600, 600)),
            (DisplayContext::PlayerPreview, (720, 405)),
        ];
        
        for (context, target_size) in contexts_and_sizes {
            match self.create_variant_for_context(
                &image_data,
                original_width,
                original_height,
                target_size,
                context,
                video_aspect_ratio,
            ).await {
                Ok(variant) => variants.push(variant),
                Err(e) => warn!("⚠️ Failed to create variant for {:?}: {}", context, e),
            }
        }
        
        // Also keep the original as a variant
        if let Ok(original_variant) = self.create_original_variant(&image_data, original_width, original_height).await {
            variants.push(original_variant);
        }
        
        Ok(variants)
    }
    
    /// Download thumbnail data from URL
    async fn download_thumbnail_data(&self, url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let response = self.client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://www.facebook.com/")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }
        
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
    
    /// Detect image dimensions from raw data
    fn detect_image_dimensions(&self, data: &[u8]) -> Result<(u32, u32), Box<dyn std::error::Error>> {
        // Simple JPEG dimension detection
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return self.detect_jpeg_dimensions(data);
        }
        
        // Simple PNG dimension detection
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return self.detect_png_dimensions(data);
        }
        
        // Default fallback
        Ok((640, 360))
    }
    
    /// Detect JPEG dimensions
    fn detect_jpeg_dimensions(&self, data: &[u8]) -> Result<(u32, u32), Box<dyn std::error::Error>> {
        // Look for SOF (Start of Frame) markers
        for i in 0..data.len().saturating_sub(9) {
            if data[i] == 0xFF && (data[i + 1] == 0xC0 || data[i + 1] == 0xC2) {
                let height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                let width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                return Ok((width, height));
            }
        }
        Err("Could not detect JPEG dimensions".into())
    }
    
    /// Detect PNG dimensions
    fn detect_png_dimensions(&self, data: &[u8]) -> Result<(u32, u32), Box<dyn std::error::Error>> {
        if data.len() >= 24 {
            let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
            let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
            return Ok((width, height));
        }
        Err("Could not detect PNG dimensions".into())
    }
    
    /// Create a variant for a specific display context
    async fn create_variant_for_context(
        &self,
        image_data: &[u8],
        original_width: u32,
        original_height: u32,
        target_size: (u32, u32),
        context: DisplayContext,
        video_aspect_ratio: AspectRatio,
    ) -> Result<ThumbnailVariant, Box<dyn std::error::Error>> {
        // For now, we'll use the original image data and create a data URL
        // In a full implementation, you would resize/crop the image here
        let data_url = self.create_data_url(image_data)?;
        
        // Calculate optimal dimensions based on context and video aspect ratio
        let (width, height) = self.calculate_optimal_dimensions(
            original_width,
            original_height,
            target_size,
            context,
            video_aspect_ratio,
        );
        
        let variant = ThumbnailVariant::new(
            width,
            height,
            data_url,
            "jpeg".to_string(),
            vec![context],
        );
        
        Ok(variant)
    }
    
    /// Calculate optimal dimensions for a variant
    fn calculate_optimal_dimensions(
        &self,
        _original_width: u32,
        _original_height: u32,
        target_size: (u32, u32),
        _context: DisplayContext,
        _video_aspect_ratio: AspectRatio,
    ) -> (u32, u32) {
        let (target_width, target_height) = target_size;
        // let _target_ratio = target_width as f32 / target_height as f32;
        // let _original_ratio = original_width as f32 / original_height as f32;

        // For now, return target size
        // In a full implementation, you would calculate based on aspect ratio preservation
        (target_width, target_height)
    }
    
    /// Create original variant
    async fn create_original_variant(
        &self,
        image_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<ThumbnailVariant, Box<dyn std::error::Error>> {
        let data_url = self.create_data_url(image_data)?;
        
        let variant = ThumbnailVariant::new(
            width,
            height,
            data_url,
            "jpeg".to_string(),
            vec![DisplayContext::DesktopFullscreen, DisplayContext::PlayerPreview],
        );
        
        Ok(variant)
    }
    
    /// Create data URL from image data
    fn create_data_url(&self, data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        let mime_type = if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            "image/jpeg"
        } else if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            "image/png"
        } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
            "image/webp"
        } else {
            "image/jpeg" // Default
        };
        
        let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data);
        Ok(format!("data:{};base64,{}", mime_type, base64_data))
    }
    
    /// Generate fallback variants when no thumbnails are available
    fn generate_fallback_variants(&self, collection: &mut ThumbnailCollection) {
        let contexts = vec![
            DisplayContext::DesktopFullscreen,
            DisplayContext::MobilePortrait,
            DisplayContext::MobileLandscape,
            DisplayContext::WebThumbnail,
            DisplayContext::SocialShare,
            DisplayContext::PlayerPreview,
        ];
        
        for context in contexts {
            let (width, height) = context.preferred_dimensions();
            let svg_data_url = self.create_fallback_svg(width, height);
            
            let variant = ThumbnailVariant::new(
                width,
                height,
                svg_data_url,
                "svg".to_string(),
                vec![context],
            );
            
            collection.add_variant(variant);
        }
    }
    
    /// Create fallback SVG thumbnail
    fn create_fallback_svg(&self, width: u32, height: u32) -> String {
        let svg_content = format!(
            "<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"100%\" height=\"100%\" fill=\"#f3f4f6\"/><circle cx=\"50%\" cy=\"40%\" r=\"20\" fill=\"#9ca3af\"/><polygon points=\"45,35 45,45 55,40\" fill=\"#f3f4f6\"/><text x=\"50%\" y=\"65%\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"12\" fill=\"#6b7280\">No Thumbnail</text></svg>",
            width, height
        );
        
        format!(
            "data:image/svg+xml;base64,{}",
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, svg_content.as_bytes())
        )
    }
}

impl Default for ThumbnailExtractor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback with basic client
            Self {
                client: Client::new(),
            }
        })
    }
}
