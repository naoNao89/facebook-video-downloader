//! Data types and structures for Facebook video extraction

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Type of Facebook content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FacebookContentType {
    /// Regular Facebook video
    RegularVideo,
    /// Facebook Reel
    Reel,
    /// Facebook Story
    Story,
    /// Live video stream
    LiveVideo,
    /// Private video content
    PrivateVideo,
}

/// Privacy level of the content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacyLevel {
    /// Public content
    Public,
    /// Friends only
    Friends,
    /// Private/restricted
    Private,
    /// Unknown privacy level
    Unknown,
}

/// Method used to access the content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessMethod {
    /// Direct URL access
    Direct,
    /// Mobile version access
    Mobile,
    /// View-source method
    ViewSource,
    /// CORS proxy access
    CorsProxy,
    /// Alternative public extraction method
    Alternative,
}

/// Stream type classification for Facebook videos
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamType {
    /// Complete video with audio (progressive streams)
    CompleteVideoAudio,
    /// Video-only stream (DASH)
    VideoOnly,
    /// Audio-only stream (DASH)
    AudioOnly,
    /// Combined video+audio stream (created via FFmpeg)
    CombinedVideoAudio,
    /// Unknown or unclassified stream
    Unknown,
}

impl StreamType {
    /// Check if stream contains video
    pub fn has_video(&self) -> bool {
        matches!(
            self,
            Self::CompleteVideoAudio | Self::VideoOnly | Self::CombinedVideoAudio
        )
    }

    /// Check if stream contains audio
    pub fn has_audio(&self) -> bool {
        matches!(
            self,
            Self::CompleteVideoAudio | Self::AudioOnly | Self::CombinedVideoAudio
        )
    }

    /// Check if stream is complete (has both video and audio)
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
    pub efg_metadata: String,
    pub estimated_size_mb: u32,
    pub bitrate_kbps: Option<u32>,
    pub fps: Option<u32>,
    pub codec: Option<String>,
}

impl VideoQuality {
    /// Calculate pixel count for quality comparison
    pub fn pixel_count(&self) -> u32 {
        self.width * self.height
    }

    /// Get quality score for sorting (higher is better)
    pub fn quality_score(&self) -> u32 {
        let pixel_score = self.pixel_count();
        let bitrate_score = self.bitrate_kbps.unwrap_or(0);
        let complete_bonus = if self.stream_type.is_complete() {
            1000000
        } else {
            0
        };

        pixel_score + bitrate_score + complete_bonus
    }

    /// Check if this quality is better than another
    pub fn is_better_than(&self, other: &VideoQuality) -> bool {
        self.quality_score() > other.quality_score()
    }
}

/// Aspect ratio for thumbnails and videos
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AspectRatio {
    /// 16:9 landscape (1.78:1) - standard widescreen
    Landscape16x9,
    /// 9:16 portrait (0.56:1) - mobile vertical
    Portrait9x16,
    /// 4:3 traditional (1.33:1) - older content
    Traditional4x3,
    /// 1:1 square - social media
    Square1x1,
    /// Custom aspect ratio
    Custom(f32),
}

impl AspectRatio {
    /// Calculate aspect ratio from dimensions
    pub fn from_dimensions(width: u32, height: u32) -> Self {
        if width == 0 || height == 0 {
            return Self::Landscape16x9; // Default fallback
        }

        let ratio = width as f32 / height as f32;

        // Use tolerance for floating point comparison
        const TOLERANCE: f32 = 0.05;

        if (ratio - 16.0/9.0).abs() < TOLERANCE {
            Self::Landscape16x9
        } else if (ratio - 9.0/16.0).abs() < TOLERANCE {
            Self::Portrait9x16
        } else if (ratio - 4.0/3.0).abs() < TOLERANCE {
            Self::Traditional4x3
        } else if (ratio - 1.0).abs() < TOLERANCE {
            Self::Square1x1
        } else {
            Self::Custom(ratio)
        }
    }

    /// Get the numeric ratio value
    pub fn ratio(&self) -> f32 {
        match self {
            Self::Landscape16x9 => 16.0 / 9.0,
            Self::Portrait9x16 => 9.0 / 16.0,
            Self::Traditional4x3 => 4.0 / 3.0,
            Self::Square1x1 => 1.0,
            Self::Custom(ratio) => *ratio,
        }
    }

    /// Check if this is a landscape orientation
    pub fn is_landscape(&self) -> bool {
        self.ratio() > 1.0
    }

    /// Check if this is a portrait orientation
    pub fn is_portrait(&self) -> bool {
        self.ratio() < 1.0
    }

    /// Check if this is square
    pub fn is_square(&self) -> bool {
        (self.ratio() - 1.0).abs() < 0.01
    }
}

/// Display context for thumbnail selection
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DisplayContext {
    /// Desktop fullscreen viewing
    DesktopFullscreen,
    /// Mobile portrait mode
    MobilePortrait,
    /// Mobile landscape mode
    MobileLandscape,
    /// Web thumbnail (small preview)
    WebThumbnail,
    /// Social media sharing
    SocialShare,
    /// Video player preview
    PlayerPreview,
}

impl DisplayContext {
    /// Get preferred aspect ratio for this display context
    pub fn preferred_aspect_ratio(&self) -> AspectRatio {
        match self {
            Self::DesktopFullscreen => AspectRatio::Landscape16x9,
            Self::MobilePortrait => AspectRatio::Portrait9x16,
            Self::MobileLandscape => AspectRatio::Landscape16x9,
            Self::WebThumbnail => AspectRatio::Landscape16x9,
            Self::SocialShare => AspectRatio::Square1x1,
            Self::PlayerPreview => AspectRatio::Landscape16x9,
        }
    }

    /// Get preferred dimensions for this display context
    pub fn preferred_dimensions(&self) -> (u32, u32) {
        match self {
            Self::DesktopFullscreen => (1920, 1080),
            Self::MobilePortrait => (720, 1280),
            Self::MobileLandscape => (1280, 720),
            Self::WebThumbnail => (480, 270),
            Self::SocialShare => (600, 600),
            Self::PlayerPreview => (720, 405),
        }
    }
}

/// Individual thumbnail variant with specific dimensions and quality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailVariant {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Aspect ratio
    pub aspect_ratio: AspectRatio,
    /// Data URL (base64 encoded image)
    pub data_url: String,
    /// File size in bytes
    pub file_size: usize,
    /// Image format (jpeg, png, webp)
    pub format: String,
    /// Quality score (0-100, higher is better)
    pub quality_score: u32,
    /// Display contexts this variant is optimized for
    pub optimized_for: Vec<DisplayContext>,
}

impl ThumbnailVariant {
    /// Create a new thumbnail variant
    pub fn new(
        width: u32,
        height: u32,
        data_url: String,
        format: String,
        optimized_for: Vec<DisplayContext>,
    ) -> Self {
        let aspect_ratio = AspectRatio::from_dimensions(width, height);
        let file_size = Self::estimate_file_size(&data_url);
        let quality_score = Self::calculate_quality_score(width, height, &format);

        Self {
            width,
            height,
            aspect_ratio,
            data_url,
            file_size,
            format,
            quality_score,
            optimized_for,
        }
    }

    /// Estimate file size from data URL
    fn estimate_file_size(data_url: &str) -> usize {
        if data_url.starts_with("data:") {
            // Base64 encoding adds ~33% overhead, so actual size is ~75% of encoded size
            let base64_part = data_url.split(',').nth(1).unwrap_or("");
            (base64_part.len() * 3) / 4
        } else {
            0
        }
    }

    /// Calculate quality score based on dimensions and format
    fn calculate_quality_score(width: u32, height: u32, format: &str) -> u32 {
        let pixel_score = (width * height) / 1000; // Scale down for reasonable numbers
        let format_bonus = match format.to_lowercase().as_str() {
            "webp" => 20,
            "png" => 15,
            "jpeg" | "jpg" => 10,
            _ => 5,
        };

        pixel_score + format_bonus
    }

    /// Check if this variant is suitable for a display context
    pub fn is_suitable_for(&self, context: DisplayContext) -> bool {
        self.optimized_for.contains(&context) || self.aspect_ratio == context.preferred_aspect_ratio()
    }

    /// Get pixel count
    pub fn pixel_count(&self) -> u32 {
        self.width * self.height
    }
}

/// Collection of thumbnail variants for different display contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailCollection {
    /// All available thumbnail variants
    pub variants: Vec<ThumbnailVariant>,
    /// Original video aspect ratio (detected from video streams)
    pub original_aspect_ratio: AspectRatio,
    /// Default thumbnail (for backward compatibility)
    pub default_thumbnail: String,
}

impl ThumbnailCollection {
    /// Create a new thumbnail collection
    pub fn new(original_aspect_ratio: AspectRatio) -> Self {
        Self {
            variants: Vec::new(),
            original_aspect_ratio,
            default_thumbnail: String::new(),
        }
    }

    /// Add a thumbnail variant
    pub fn add_variant(&mut self, variant: ThumbnailVariant) {
        // Update default thumbnail if this is the first or highest quality variant
        if self.default_thumbnail.is_empty() ||
           variant.quality_score > self.get_best_variant().map(|v| v.quality_score).unwrap_or(0) {
            self.default_thumbnail = variant.data_url.clone();
        }

        self.variants.push(variant);
    }

    /// Get the best variant for a specific display context
    pub fn get_variant_for_context(&self, context: DisplayContext) -> Option<&ThumbnailVariant> {
        // First, try to find variants optimized for this context
        let mut suitable_variants: Vec<_> = self.variants
            .iter()
            .filter(|v| v.is_suitable_for(context))
            .collect();

        if suitable_variants.is_empty() {
            // If no suitable variants, try to find variants with matching aspect ratio
            suitable_variants = self.variants
                .iter()
                .filter(|v| v.aspect_ratio == context.preferred_aspect_ratio())
                .collect();
        }

        if suitable_variants.is_empty() {
            // Fallback to any variant
            suitable_variants = self.variants.iter().collect();
        }

        // Sort by quality score and return the best one
        suitable_variants.sort_by(|a, b| b.quality_score.cmp(&a.quality_score));
        suitable_variants.first().copied()
    }

    /// Get the highest quality variant overall
    pub fn get_best_variant(&self) -> Option<&ThumbnailVariant> {
        self.variants.iter().max_by_key(|v| v.quality_score)
    }

    /// Get variants by aspect ratio
    pub fn get_variants_by_aspect_ratio(&self, aspect_ratio: AspectRatio) -> Vec<&ThumbnailVariant> {
        self.variants
            .iter()
            .filter(|v| v.aspect_ratio == aspect_ratio)
            .collect()
    }

    /// Get all available aspect ratios
    pub fn available_aspect_ratios(&self) -> Vec<AspectRatio> {
        let mut ratios: Vec<_> = self.variants
            .iter()
            .map(|v| v.aspect_ratio)
            .collect();
        ratios.sort_by(|a, b| a.ratio().partial_cmp(&b.ratio()).unwrap_or(std::cmp::Ordering::Equal));
        ratios.dedup();
        ratios
    }

    /// Check if collection has variants for a specific context
    pub fn has_variant_for_context(&self, context: DisplayContext) -> bool {
        self.get_variant_for_context(context).is_some()
    }

    /// Get summary of available variants
    pub fn get_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();

        for variant in &self.variants {
            let key = format!("{}x{}", variant.width, variant.height);
            *summary.entry(key).or_insert(0) += 1;
        }

        summary
    }
}

impl Default for ThumbnailCollection {
    fn default() -> Self {
        Self::new(AspectRatio::Landscape16x9)
    }
}

/// Enhanced video metadata with structured data
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
    pub author_url: Option<String>,
    pub author_verified: bool,
    pub privacy_level: Option<PrivacyLevel>,
    pub location: Option<String>,
    pub content_warnings: Vec<String>,
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
            author_url: None,
            author_verified: false,
            privacy_level: None,
            location: None,
            content_warnings: Vec::new(),
        }
    }
}

/// Complete video information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub title: String,
    pub duration: String,
    /// Legacy thumbnail field (for backward compatibility)
    pub thumbnail: String,
    /// Enhanced thumbnail collection with multiple variants
    pub thumbnail_variants: ThumbnailCollection,
    pub qualities: Vec<VideoQuality>,
    pub video_id: String,
    pub metadata: VideoMetadata,
    pub extraction_timestamp: DateTime<Utc>,
    pub source_url: String,
    pub content_type: FacebookContentType,
    pub privacy_level: PrivacyLevel,
    pub access_method: AccessMethod,
}

impl VideoInfo {
    /// Create new video info with current timestamp
    pub fn new(title: String, video_id: String, source_url: String) -> Self {
        Self {
            title,
            duration: "Unknown duration".to_string(),
            thumbnail: String::new(),
            thumbnail_variants: ThumbnailCollection::default(),
            qualities: Vec::new(),
            video_id,
            metadata: VideoMetadata::default(),
            extraction_timestamp: Utc::now(),
            source_url,
            content_type: FacebookContentType::RegularVideo,
            privacy_level: PrivacyLevel::Unknown,
            access_method: AccessMethod::Direct,
        }
    }

    /// Get the best quality stream available
    pub fn best_quality(&self) -> Option<&VideoQuality> {
        self.qualities.iter().max_by_key(|q| q.quality_score())
    }

    /// Get all complete streams (video+audio)
    pub fn complete_streams(&self) -> Vec<&VideoQuality> {
        self.qualities
            .iter()
            .filter(|q| q.stream_type.is_complete())
            .collect()
    }

    /// Get video-only streams
    pub fn video_only_streams(&self) -> Vec<&VideoQuality> {
        self.qualities
            .iter()
            .filter(|q| q.stream_type == StreamType::VideoOnly)
            .collect()
    }

    /// Get audio-only streams
    pub fn audio_only_streams(&self) -> Vec<&VideoQuality> {
        self.qualities
            .iter()
            .filter(|q| q.stream_type == StreamType::AudioOnly)
            .collect()
    }

    /// Detect video aspect ratio from available streams
    pub fn detect_video_aspect_ratio(&self) -> AspectRatio {
        // Try to get aspect ratio from the best quality stream
        if let Some(best_quality) = self.best_quality() {
            return AspectRatio::from_dimensions(best_quality.width, best_quality.height);
        }

        // Fallback to any stream with dimensions
        for quality in &self.qualities {
            if quality.width > 0 && quality.height > 0 {
                return AspectRatio::from_dimensions(quality.width, quality.height);
            }
        }

        // Default fallback
        AspectRatio::Landscape16x9
    }

    /// Get thumbnail for specific display context
    pub fn get_thumbnail_for_context(&self, context: DisplayContext) -> Option<&ThumbnailVariant> {
        self.thumbnail_variants.get_variant_for_context(context)
    }

    /// Get the best available thumbnail
    pub fn get_best_thumbnail(&self) -> Option<&ThumbnailVariant> {
        self.thumbnail_variants.get_best_variant()
    }

    /// Get thumbnail data URL for specific context (with fallback)
    pub fn get_thumbnail_url_for_context(&self, context: DisplayContext) -> String {
        if let Some(variant) = self.get_thumbnail_for_context(context) {
            variant.data_url.clone()
        } else if !self.thumbnail.is_empty() {
            self.thumbnail.clone()
        } else {
            // Return a fallback SVG
            self.create_fallback_thumbnail_svg()
        }
    }

    /// Create a fallback SVG thumbnail
    fn create_fallback_thumbnail_svg(&self) -> String {
        let (width, height) = DisplayContext::WebThumbnail.preferred_dimensions();
        let svg_content = format!(
            "<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"100%\" height=\"100%\" fill=\"#f3f4f6\"/><text x=\"50%\" y=\"50%\" text-anchor=\"middle\" dy=\"0.3em\" font-family=\"Arial, sans-serif\" font-size=\"14\" fill=\"#6b7280\">No Thumbnail</text></svg>",
            width, height
        );

        format!(
            "data:image/svg+xml;base64,{}",
            base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                svg_content.as_bytes()
            )
        )
    }

    /// Update thumbnail variants and sync legacy thumbnail field
    pub fn set_thumbnail_variants(&mut self, variants: ThumbnailCollection) {
        self.thumbnail = variants.default_thumbnail.clone();
        self.thumbnail_variants = variants;
    }

    /// Add a single thumbnail variant
    pub fn add_thumbnail_variant(&mut self, variant: ThumbnailVariant) {
        self.thumbnail_variants.add_variant(variant);
        // Update legacy thumbnail field
        self.thumbnail = self.thumbnail_variants.default_thumbnail.clone();
    }
}

/// URL validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlValidation {
    pub is_valid: bool,
    pub content_type: Option<FacebookContentType>,
    pub video_id: Option<String>,
    pub error_message: Option<String>,
    pub suggestions: Vec<String>,
}

impl UrlValidation {
    /// Create a valid URL validation result
    pub fn valid(content_type: FacebookContentType, video_id: String) -> Self {
        Self {
            is_valid: true,
            content_type: Some(content_type),
            video_id: Some(video_id),
            error_message: None,
            suggestions: Vec::new(),
        }
    }

    /// Create an invalid URL validation result
    pub fn invalid(error_message: String, suggestions: Vec<String>) -> Self {
        Self {
            is_valid: false,
            content_type: None,
            video_id: None,
            error_message: Some(error_message),
            suggestions,
        }
    }
}
