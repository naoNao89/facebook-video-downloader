use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use regex::Regex;
use chrono::Utc;
use reqwest;
use std::{time::Duration, path::Path};
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use std::time::Instant;
use base64::Engine;
use facebook_video_downloader_core::{
    processing::{CompressionService, CompressionQuality, CompressionOptions, CompressionProgress, CompressionResult},
    batch::{BatchProcessor, BatchJob, BatchProgress, BatchOptions},
    FacebookExtractor, FacebookExtractorError, VideoInfo as CoreVideoInfo, VideoQuality as CoreVideoQuality,
    VideoMetadata as CoreVideoMetadata, common::StreamType as CoreStreamType
};

// ============================================================================
// TYPE ALIASES
// ============================================================================

// Use the error type from the core crate
type ExtractorResult<T> = std::result::Result<T, FacebookExtractorError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriResult<T> {
    pub data: T,
    pub message: Option<String>,
}

// ============================================================================
// DOWNLOAD PROGRESS TRACKING
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub id: String,
    pub progress: f64, // 0.0 to 100.0
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed_bytes_per_sec: Option<u64>,
    pub eta_seconds: Option<u64>,
    pub status: DownloadStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Queued,
    Starting,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlValidation {
    pub is_valid: bool,
    pub content_type: Option<String>,
    pub video_id: Option<String>,
    pub error_message: Option<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub title: String,
    pub duration: String,
    pub thumbnail: String,
    pub qualities: Vec<VideoQuality>,
    pub video_id: String,
    pub metadata: VideoMetadata,
    pub extraction_timestamp: String,
    pub source_url: String,
    pub content_type: String,
    pub privacy_level: String,
    pub access_method: String,
}

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
    pub privacy_level: Option<String>,
    pub location: Option<String>,
    pub content_warnings: Vec<String>,
}

impl VideoMetadata {
    pub fn default() -> Self {
        Self {
            author: "Unknown Author".to_string(),
            description: "No description available".to_string(),
            publish_date: Utc::now().format("%Y-%m-%d").to_string(),
            likes: 0,
            comments: 0,
            views: 0,
            shares: 0,
            hashtags: vec![],
            duration_seconds: None,
            language: Some("en".to_string()),
            category: None,
            author_url: None,
            author_verified: false,
            privacy_level: Some("public".to_string()),
            location: None,
            content_warnings: vec![],
        }
    }
}

impl VideoInfo {
    pub fn new(title: String, video_id: String, source_url: String) -> Self {
        Self {
            title,
            duration: "Unknown".to_string(),
            thumbnail: "https://via.placeholder.com/640x360".to_string(),
            qualities: vec![],
            video_id,
            metadata: VideoMetadata::default(),
            extraction_timestamp: Utc::now().to_rfc3339(),
            source_url,
            content_type: "RegularVideo".to_string(),
            privacy_level: "Public".to_string(),
            access_method: "Direct".to_string(),
        }
    }
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

// Simple test command to verify Tauri API is working
#[tauri::command]
async fn test_tauri_connection() -> Result<TauriResult<String>, String> {
    Ok(TauriResult {
        data: "Tauri API is working correctly!".to_string(),
        message: Some("Connection test successful".to_string()),
    })
}

// Get app version
#[tauri::command]
async fn get_app_version() -> Result<TauriResult<String>, String> {
    Ok(TauriResult {
        data: env!("CARGO_PKG_VERSION").to_string(),
        message: Some("App version retrieved".to_string()),
    })
}

// Read clipboard content
#[tauri::command]
async fn read_clipboard(app: tauri::AppHandle) -> Result<TauriResult<String>, String> {
    tracing::info!("📋 Reading clipboard content");

    match app.clipboard().read_text() {
        Ok(content) => {
            tracing::info!("✅ Clipboard content read successfully: {} chars", content.len());
            Ok(TauriResult {
                data: content,
                message: Some("Clipboard content read successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ Failed to read clipboard: {}", e);
            Err(format!("Failed to read clipboard: {}", e))
        }
    }
}

// Write content to clipboard
#[tauri::command]
async fn write_clipboard(app: tauri::AppHandle, text: String) -> Result<TauriResult<()>, String> {
    tracing::info!("📋 Writing to clipboard: {} chars", text.len());

    match app.clipboard().write_text(text) {
        Ok(_) => {
            tracing::info!("✅ Content written to clipboard successfully");
            Ok(TauriResult {
                data: (),
                message: Some("Content copied to clipboard successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ Failed to write to clipboard: {}", e);
            Err(format!("Failed to write to clipboard: {}", e))
        }
    }
}

// Validate Facebook URL
#[tauri::command]
async fn validate_facebook_url(url: String) -> Result<TauriResult<UrlValidation>, String> {
    tracing::info!("🔍 Validating Facebook URL: {}", url);

    let patterns = [
        r"^https?://(?:www\.)?facebook\.com/watch/?\?.*v=\d+",  // Fixed: handle both /watch? and /watch/?
        r"^https?://(?:www\.)?facebook\.com/[^/]+/videos/\d+",
        r"^https?://fb\.watch/[a-zA-Z0-9_-]+",
        r"^https?://(?:www\.)?facebook\.com/video\.php\?v=\d+",
        r"^https?://(?:www\.)?facebook\.com/[^/]+/posts/\d+",
        r"^https?://(?:www\.)?facebook\.com/reel/\d+",
        r"^https?://(?:www\.)?facebook\.com/share/v/[a-zA-Z0-9_-]+/?",  // New sharing format
        r"^https?://(?:www\.)?facebook\.com/share/r/[a-zA-Z0-9_-]+/?",  // New sharing format for reels
    ];

    let mut is_valid = false;
    let mut url_type = None;

    for (i, pattern) in patterns.iter().enumerate() {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(&url) {
                is_valid = true;
                url_type = Some(match i {
                    0 => "watch",
                    1 => "video",
                    2 => "fb.watch",
                    3 => "video.php",
                    4 => "post",
                    5 => "reel",
                    6 => "share_video",  // New sharing format for videos
                    7 => "share_reel",   // New sharing format for reels
                    _ => "unknown",
                }.to_string());
                break;
            }
        }
    }

    // Extract video ID if URL is valid
    let video_id = if is_valid {
        match FacebookExtractor::new() {
            Ok(extractor) => extractor.extract_video_id(&url).ok(),
            Err(_) => None,
        }
    } else {
        None
    };

    let validation = UrlValidation {
        is_valid,
        content_type: url_type,
        video_id,
        error_message: if !is_valid {
            Some("Invalid Facebook URL format. Please provide a valid Facebook video URL.".to_string())
        } else {
            None
        },
        suggestions: if !is_valid {
            vec![
                "URL must contain 'facebook.com'".to_string(),
                "Supported formats: /watch?v=ID, /videos/ID, /reel/ID, /share/v/ID".to_string(),
            ]
        } else {
            vec![]
        },
    };

    tracing::info!("✅ URL validation result: valid={}, type={:?}, video_id={:?}", validation.is_valid, validation.content_type, validation.video_id);

    Ok(TauriResult {
        data: validation,
        message: Some("URL validation completed".to_string()),
    })
}

// Extract video info using the enhanced Facebook extractor
#[tauri::command]
async fn extract_video_info(url: String) -> Result<TauriResult<VideoInfo>, String> {
    tracing::info!("🎬 [BACKEND] Starting real Facebook video extraction for URL: {}", url);
    tracing::info!("🔍 [BACKEND] Tauri command extract_video_info received and processing...");

    // Use the enhanced extractor from facebook-extractor-core
    let extractor = FacebookExtractor::new()
        .map_err(|e| {
            tracing::error!("❌ [BACKEND] Failed to create extractor: {}", e);
            format!("Failed to create extractor: {}", e)
        })?;

    tracing::info!("✅ [BACKEND] Extractor created successfully, starting extraction...");

    // Extract video information
    match extractor.extract_video_info(&url).await {
        Ok(core_video_info) => {
            tracing::info!("✅ [BACKEND] Video extraction successful for video ID: {}", core_video_info.video_id);
            tracing::info!("📋 [BACKEND] Found {} video qualities", core_video_info.qualities.len());

            // Convert from core VideoInfo to Tauri VideoInfo
            let video_info = convert_core_video_info(core_video_info);

            tracing::info!("🔄 [BACKEND] Converted video info, preparing response...");
            tracing::info!("✅ [BACKEND] About to return successful TauriResult");

            Ok(TauriResult {
                data: video_info,
                message: Some("Video information extracted successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ [BACKEND] Video extraction failed: {}", e);

            // Return a more detailed error message
            let error_msg = match e {
                FacebookExtractorError::InvalidUrl { url: _ } => "Invalid Facebook URL format. Please provide a valid Facebook video URL.".to_string(),
                FacebookExtractorError::VideoIdExtraction { message: _ } => "Could not extract video ID from the provided URL.".to_string(),
                FacebookExtractorError::Network { source: _ } => "Network error occurred while fetching video data.".to_string(),
                FacebookExtractorError::HtmlParsing { message: _ } => "Failed to parse video information from Facebook page.".to_string(),
                FacebookExtractorError::StreamAnalysis { message: _ } => "No video streams found on this page.".to_string(),
                FacebookExtractorError::Timeout { message: _ } => "Request timed out while fetching video data.".to_string(),
                FacebookExtractorError::Download { message: _ } => "Download error occurred.".to_string(),
                FacebookExtractorError::AccessDenied { reason: _ } => "Video access denied.".to_string(),
                FacebookExtractorError::AuthenticationRequired => "Authentication required to access this video.".to_string(),
                FacebookExtractorError::GeoBlocked => "This video is not available in your region.".to_string(),
                FacebookExtractorError::ContentUnavailable => "This video has been removed or is no longer available.".to_string(),
                FacebookExtractorError::RateLimited => "Rate limit exceeded. Please try again later.".to_string(),
                _ => format!("Extraction failed: {}", e),
            };

            Err(error_msg)
        }
    }
}

// Convert from core VideoInfo to Tauri VideoInfo
fn convert_core_video_info(core_info: CoreVideoInfo) -> VideoInfo {
    VideoInfo {
        title: core_info.title,
        duration: core_info.duration,
        thumbnail: core_info.thumbnail,
        qualities: core_info.qualities.into_iter().map(convert_core_quality).collect(),
        video_id: core_info.video_id,
        metadata: convert_core_metadata(core_info.metadata),
        extraction_timestamp: core_info.extraction_timestamp.to_rfc3339(),
        source_url: core_info.source_url,
        content_type: "RegularVideo".to_string(),
        privacy_level: "Public".to_string(),
        access_method: "Direct".to_string(),
    }
}

fn convert_core_quality(core_quality: CoreVideoQuality) -> VideoQuality {
    VideoQuality {
        quality: core_quality.quality,
        size: core_quality.size,
        format: core_quality.format,
        download_url: core_quality.download_url,
        width: core_quality.width,
        height: core_quality.height,
        stream_type: convert_core_stream_type(core_quality.stream_type),
        efg_metadata: core_quality.efg_metadata,
        estimated_size_mb: core_quality.estimated_size_mb,
        bitrate_kbps: core_quality.bitrate_kbps,
        fps: core_quality.fps,
        codec: core_quality.codec,
    }
}

fn convert_core_stream_type(core_type: CoreStreamType) -> StreamType {
    match core_type {
        CoreStreamType::CompleteVideoAudio => StreamType::CompleteVideoAudio,
        CoreStreamType::VideoOnly => StreamType::VideoOnly,
        CoreStreamType::AudioOnly => StreamType::AudioOnly,
        CoreStreamType::CombinedVideoAudio => StreamType::CombinedVideoAudio,
        CoreStreamType::Unknown => StreamType::Unknown,
    }
}

fn convert_core_metadata(core_metadata: CoreVideoMetadata) -> VideoMetadata {
    VideoMetadata {
        author: core_metadata.author,
        description: core_metadata.description,
        publish_date: core_metadata.publish_date,
        likes: core_metadata.likes,
        comments: core_metadata.comments,
        views: core_metadata.views,
        shares: core_metadata.shares,
        hashtags: core_metadata.hashtags,
        duration_seconds: core_metadata.duration_seconds,
        language: core_metadata.language,
        category: core_metadata.category,
        author_url: None,
        author_verified: false,
        privacy_level: Some("public".to_string()),
        location: None,
        content_warnings: vec![],
    }
}

// Enhanced thumbnail fetcher with Facebook-specific headers and fallback strategies
#[tauri::command]
async fn fetch_thumbnail_image(url: String) -> Result<TauriResult<String>, String> {
    tracing::info!("🖼️ Enhanced thumbnail fetching from: {}", &url[..100.min(url.len())]);

    // Check if the URL is already a data URL (from fallback SVG)
    if url.starts_with("data:") {
        tracing::info!("✅ URL is already a data URL - returning as-is");
        return Ok(TauriResult {
            data: url,
            message: Some("Data URL returned directly".to_string()),
        });
    }

    // For HTTP URLs, try multiple strategies to fetch the thumbnail
    match fetch_thumbnail_with_strategies(&url).await {
        Ok(data_url) => {
            tracing::info!("✅ Thumbnail fetched successfully");
            Ok(TauriResult {
                data: data_url,
                message: Some("Thumbnail fetched successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ All thumbnail fetch strategies failed: {}", e);

            // Return a placeholder data URL instead of failing completely
            let placeholder_data_url = create_placeholder_thumbnail();
            tracing::info!("🔄 Using placeholder thumbnail as fallback");

            Ok(TauriResult {
                data: placeholder_data_url,
                message: Some(format!("Using placeholder (original failed: {})", e)),
            })
        }
    }
}

// Try multiple strategies to fetch thumbnail
async fn fetch_thumbnail_with_strategies(url: &str) -> Result<String, String> {
    const MAX_RETRIES: usize = 3;

    // Strategy 1: Enhanced Facebook headers with session simulation
    tracing::info!("🔄 Strategy 1: Enhanced Facebook headers");
    match fetch_thumbnail_with_facebook_headers(url, MAX_RETRIES).await {
        Ok(data_url) => {
            tracing::info!("✅ Strategy 1 succeeded");
            return Ok(data_url);
        }
        Err(e) => {
            tracing::warn!("⚠️ Strategy 1 failed: {}", e);
        }
    }

    // Strategy 2: Alternative user agents and headers
    tracing::info!("🔄 Strategy 2: Alternative user agents");
    match fetch_thumbnail_with_alternative_headers(url, MAX_RETRIES).await {
        Ok(data_url) => {
            tracing::info!("✅ Strategy 2 succeeded");
            return Ok(data_url);
        }
        Err(e) => {
            tracing::warn!("⚠️ Strategy 2 failed: {}", e);
        }
    }

    // Strategy 3: Direct fetch with minimal headers
    tracing::info!("🔄 Strategy 3: Minimal headers");
    match fetch_thumbnail_minimal(url, MAX_RETRIES).await {
        Ok(data_url) => {
            tracing::info!("✅ Strategy 3 succeeded");
            return Ok(data_url);
        }
        Err(e) => {
            tracing::warn!("⚠️ Strategy 3 failed: {}", e);
        }
    }

    Err("All thumbnail fetch strategies failed".to_string())
}

// Strategy 1: Enhanced Facebook headers with session simulation
async fn fetch_thumbnail_with_facebook_headers(url: &str, max_retries: usize) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    for attempt in 1..=max_retries {
        tracing::info!("📡 Facebook headers attempt {}/{}", attempt, max_retries);

        let response = client
            .get(url)
            // Simulate a real Facebook session
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Referer", "https://www.facebook.com/")
            .header("Origin", "https://www.facebook.com")
            .header("Sec-Fetch-Dest", "image")
            .header("Sec-Fetch-Mode", "no-cors")
            .header("Sec-Fetch-Site", "same-site")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            // Add some Facebook-specific headers
            .header("X-Requested-With", "XMLHttpRequest")
            .send()
            .await;

        match response {
            Ok(resp) => {
                tracing::info!("📊 Response status: {}", resp.status());

                if resp.status().is_success() {
                    match process_thumbnail_response(resp).await {
                        Ok(data_url) => return Ok(data_url),
                        Err(e) => {
                            tracing::warn!("⚠️ Failed to process response: {}", e);
                            if attempt < max_retries {
                                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                continue;
                            }
                        }
                    }
                } else {
                    tracing::warn!("⚠️ HTTP error {}: {}", resp.status(), resp.status().canonical_reason().unwrap_or("Unknown"));
                    if attempt < max_retries {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                }
            }
            Err(e) => {
                tracing::warn!("⚠️ Network error: {}", e);
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
            }
        }
    }

    Err("Facebook headers strategy failed after all retries".to_string())
}

// Strategy 2: Alternative user agents and headers
async fn fetch_thumbnail_with_alternative_headers(url: &str, max_retries: usize) -> Result<String, String> {
    let user_agents = [
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    ];

    for (i, user_agent) in user_agents.iter().enumerate() {
        tracing::info!("🔄 Trying user agent {}/{}: {}", i + 1, user_agents.len(), &user_agent[..50]);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        for attempt in 1..=max_retries {
            let response = client
                .get(url)
                .header("User-Agent", *user_agent)
                .header("Accept", "image/*,*/*;q=0.8")
                .header("Accept-Language", "en-US,en;q=0.5")
                .header("Connection", "keep-alive")
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    match process_thumbnail_response(resp).await {
                        Ok(data_url) => {
                            tracing::info!("✅ Alternative headers succeeded with user agent {}", i + 1);
                            return Ok(data_url);
                        }
                        Err(e) => {
                            tracing::warn!("⚠️ Failed to process response: {}", e);
                        }
                    }
                }
                Ok(resp) => {
                    tracing::warn!("⚠️ HTTP error {}", resp.status());
                }
                Err(e) => {
                    tracing::warn!("⚠️ Network error: {}", e);
                }
            }

            if attempt < max_retries {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }

    Err("Alternative headers strategy failed".to_string())
}

// Strategy 3: Minimal headers approach
async fn fetch_thumbnail_minimal(url: &str, max_retries: usize) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    for attempt in 1..=max_retries {
        tracing::info!("📡 Minimal headers attempt {}/{}", attempt, max_retries);

        let response = client
            .get(url)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                match process_thumbnail_response(resp).await {
                    Ok(data_url) => {
                        tracing::info!("✅ Minimal headers succeeded");
                        return Ok(data_url);
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ Failed to process response: {}", e);
                    }
                }
            }
            Ok(resp) => {
                tracing::warn!("⚠️ HTTP error {}", resp.status());
            }
            Err(e) => {
                tracing::warn!("⚠️ Network error: {}", e);
            }
        }

        if attempt < max_retries {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    Err("Minimal headers strategy failed".to_string())
}

// Process thumbnail response and convert to data URL
async fn process_thumbnail_response(response: reqwest::Response) -> Result<String, String> {
    let bytes = response.bytes().await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;

    tracing::info!("📊 Downloaded {} bytes", bytes.len());

    if bytes.len() == 0 {
        return Err("Downloaded thumbnail is empty (0 bytes)".to_string());
    }

    if bytes.len() < 100 {
        tracing::warn!("⚠️ Downloaded thumbnail is very small ({} bytes) - might be an error", bytes.len());
        // Log the content for debugging
        if let Ok(content) = String::from_utf8(bytes.to_vec()) {
            tracing::warn!("📄 Small response content: {}", content);
        }
        return Err("Downloaded content too small to be a valid image".to_string());
    }

    // Enhanced image format detection
    let (is_valid_image, content_type) = detect_image_format(&bytes);

    if !is_valid_image {
        tracing::warn!("⚠️ Downloaded data doesn't appear to be a valid image format");
        tracing::info!("📊 First 20 bytes: {:02X?}", &bytes[..20.min(bytes.len())]);

        // Check if it's an error message
        if let Ok(content) = String::from_utf8(bytes.to_vec()) {
            if content.to_lowercase().contains("error") || content.to_lowercase().contains("forbidden") {
                return Err(format!("Server returned error: {}", content));
            }
        }

        return Err("Downloaded content is not a valid image".to_string());
    }

    // Convert bytes to base64 data URL
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:{};base64,{}", content_type, base64_data);

    tracing::info!("✅ Successfully converted to base64 data URL (length: {})", data_url.len());
    Ok(data_url)
}

// Enhanced image format detection
fn detect_image_format(bytes: &[u8]) -> (bool, &'static str) {
    if bytes.len() < 4 {
        return (false, "application/octet-stream");
    }

    // JPEG
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
        return (true, "image/jpeg");
    }

    // PNG
    if bytes.len() >= 8 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n" {
        return (true, "image/png");
    }

    // GIF
    if bytes.len() >= 6 && (&bytes[0..6] == b"GIF87a" || &bytes[0..6] == b"GIF89a") {
        return (true, "image/gif");
    }

    // WebP
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return (true, "image/webp");
    }

    // BMP
    if bytes.len() >= 2 && &bytes[0..2] == b"BM" {
        return (true, "image/bmp");
    }

    (false, "application/octet-stream")
}

// Create a placeholder thumbnail as fallback
fn create_placeholder_thumbnail() -> String {
    // Facebook-branded SVG placeholder as data URL
    let svg_content = format!(
        r#"<svg width="320" height="180" xmlns="http://www.w3.org/2000/svg">
        <defs>
            <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" style="stop-color:{};stop-opacity:1" />
                <stop offset="100%" style="stop-color:{};stop-opacity:1" />
            </linearGradient>
        </defs>
        <rect width="320" height="180" fill="url(#bg)"/>
        <circle cx="160" cy="90" r="30" fill="white" opacity="0.9"/>
        <polygon points="150,75 150,105 175,90" fill="{}"/>
        <text x="160" y="130" font-family="Arial" font-size="12" fill="white" text-anchor="middle">Facebook Video</text>
        <text x="160" y="150" font-family="Arial" font-size="10" fill="white" text-anchor="middle" opacity="0.8">Thumbnail unavailable</text>
    </svg>"#,
        "#1877f2", "#42a5f5", "#1877f2"
    );
    let base64_svg = base64::engine::general_purpose::STANDARD.encode(svg_content.as_bytes());
    format!("data:image/svg+xml;base64,{}", base64_svg)
}

// Enhanced download video functionality with progress tracking
#[tauri::command]
async fn download_video(
    video_info: VideoInfo,
    quality_index: usize,
    output_path: Option<String>,
) -> Result<TauriResult<String>, String> {
    tracing::info!("🚀 Starting enhanced video download for: {}", video_info.title);

    if quality_index >= video_info.qualities.len() {
        return Err("Invalid quality index".to_string());
    }

    let quality = &video_info.qualities[quality_index];
    let download_url = &quality.download_url;

    // Check if this is a combined DASH stream (not supported in this simplified version)
    if download_url.starts_with("COMBINED:") {
        return Err("Combined DASH streams are not supported in this version. Please use a complete video+audio stream.".to_string());
    }

    // Determine output directory
    let output_dir = match output_path {
        Some(path) => path,
        None => {
            // Use default downloads directory
            let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
            home_dir.join("Downloads").join("FacebookVideos").to_string_lossy().to_string()
        }
    };

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Generate enhanced filename using video metadata
    let filename = create_enhanced_filename(&video_info, quality, &output_dir);
    let output_file = Path::new(&output_dir).join(&filename);

    // Check if file already exists
    if output_file.exists() {
        tracing::info!("⏭️ File already exists, skipping: {}", output_file.display());
        return Ok(TauriResult {
            data: output_file.to_string_lossy().to_string(),
            message: Some("File already exists".to_string()),
        });
    }

    // Download the video with enhanced error handling and retry logic
    match download_file_enhanced(download_url, &output_file).await {
        Ok(_) => {
            tracing::info!("✅ Download completed: {}", output_file.display());
            Ok(TauriResult {
                data: output_file.to_string_lossy().to_string(),
                message: Some("Video downloaded successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ Download failed: {}", e);
            // Clean up partial file if it exists
            if output_file.exists() {
                let _ = std::fs::remove_file(&output_file);
            }
            Err(format!("Download failed: {}", e))
        }
    }
}

// Enhanced download function with progress tracking and retry logic
async fn download_file_enhanced(url: &str, output_path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const MAX_RETRIES: usize = 3;
    const _CHUNK_SIZE: usize = 8192; // 8KB chunks for progress tracking
    const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);
    const READ_TIMEOUT: Duration = Duration::from_secs(60);

    tracing::info!("🌐 Starting enhanced download from: {}", &url[..80.min(url.len())]);

    let client = reqwest::Client::builder()
        .timeout(READ_TIMEOUT)
        .connect_timeout(CONNECTION_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    for attempt in 1..=MAX_RETRIES {
        tracing::info!("📡 Download attempt {}/{}", attempt, MAX_RETRIES);

        match try_download_with_progress(&client, url, output_path).await {
            Ok(_) => {
                tracing::info!("✅ Download completed successfully");
                return Ok(());
            }
            Err(e) => {
                tracing::warn!("⚠️ Download attempt {} failed: {}", attempt, e);

                if attempt < MAX_RETRIES {
                    let delay = Duration::from_secs(2_u64.pow(attempt as u32)); // Exponential backoff
                    tracing::info!("⏳ Retrying in {} seconds...", delay.as_secs());
                    tokio::time::sleep(delay).await;

                    // Clean up partial file before retry
                    if output_path.exists() {
                        let _ = std::fs::remove_file(output_path);
                    }
                } else {
                    return Err(format!("Download failed after {} attempts: {}", MAX_RETRIES, e).into());
                }
            }
        }
    }

    Err("Download failed after all retry attempts".into())
}

// Core download function with progress tracking
async fn try_download_with_progress(
    client: &reqwest::Client,
    url: &str,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("🔍 Analyzing download URL...");

    // Add Facebook-specific headers for better compatibility
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "*/*")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Referer", "https://www.facebook.com/")
        .header("Origin", "https://www.facebook.com")
        .header("Connection", "keep-alive")
        .header("Sec-Fetch-Dest", "video")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "cross-site")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {} - {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown")).into());
    }

    let total_size = response.content_length();
    tracing::info!("📊 File size: {} MB", total_size.map(|s| s / 1024 / 1024).unwrap_or(0));

    let mut file = tokio::fs::File::create(output_path).await?;
    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();
    let start_time = Instant::now();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        // Log progress every 5MB or at completion
        if downloaded % (5 * 1024 * 1024) == 0 || (total_size.is_some() && downloaded >= total_size.unwrap()) {
            let progress = if let Some(total) = total_size {
                (downloaded as f64 / total as f64 * 100.0) as u32
            } else {
                0
            };

            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                (downloaded as f64 / 1024.0 / 1024.0) / elapsed
            } else {
                0.0
            };

            tracing::info!(
                "📈 Progress: {} MB / {} MB ({}%) - Speed: {:.1} MB/s",
                downloaded / 1024 / 1024,
                total_size.map(|s| s / 1024 / 1024).unwrap_or(0),
                progress,
                speed
            );
        }
    }

    file.flush().await?;

    // Verify file was downloaded completely
    if let Some(expected_size) = total_size {
        if downloaded != expected_size {
            return Err(format!("Download incomplete: got {} bytes, expected {} bytes", downloaded, expected_size).into());
        }
    }

    tracing::info!("✅ Download verification passed: {} bytes", downloaded);
    Ok(())
}

// Create enhanced filename using video metadata
fn create_enhanced_filename(video_info: &VideoInfo, quality: &VideoQuality, _output_dir: &str) -> String {
    // Clean the title by removing hashtags and metadata
    let clean_title = if !video_info.title.is_empty() && video_info.title != "Unknown" {
        remove_hashtags_and_metadata(&video_info.title)
    } else {
        format!("Facebook_Video_{}", &video_info.video_id[..8.min(video_info.video_id.len())])
    };

    // Create quality suffix
    let quality_suffix = quality.quality
        .replace(" ", "_")
        .replace("(", "")
        .replace(")", "")
        .to_lowercase();

    // Create the filename
    let filename = format!("{}_{}.mp4", clean_title, quality_suffix);

    // Sanitize the filename
    sanitize_filename(&filename)
}

// Remove hashtags and metadata from title
fn remove_hashtags_and_metadata(title: &str) -> String {
    let mut clean_title = title.to_string();

    // Remove hashtags
    clean_title = clean_title.split('#').next().unwrap_or(&clean_title).to_string();

    // Remove common metadata patterns
    let patterns_to_remove = [
        " | Facebook",
        " - Facebook",
        " on Facebook",
        " · Facebook",
    ];

    for pattern in &patterns_to_remove {
        clean_title = clean_title.replace(pattern, "");
    }

    // Trim and limit length
    clean_title = clean_title.trim().chars().take(50).collect();

    if clean_title.is_empty() {
        "Facebook_Video".to_string()
    } else {
        clean_title
    }
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '|' | '?' | '*' | '\\' | '/' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

// ============================================================================
// COMPRESSION COMMANDS
// ============================================================================

/// Check if compression is available (FFmpeg installed)
#[tauri::command]
async fn check_compression_availability() -> Result<TauriResult<bool>, String> {
    tracing::info!("🔧 Checking compression availability");

    match CompressionService::new() {
        Ok(service) => {
            let available = service.is_available();
            tracing::info!("✅ Compression availability: {}", available);

            Ok(TauriResult {
                data: available,
                message: if available {
                    Some("FFmpeg is available for video compression".to_string())
                } else {
                    Some("FFmpeg not found. Please install FFmpeg to use compression features.".to_string())
                },
            })
        }
        Err(e) => {
            tracing::error!("❌ Failed to check compression availability: {}", e);
            Ok(TauriResult {
                data: false,
                message: Some(format!("Compression check failed: {}", e)),
            })
        }
    }
}

/// Get available compression quality options
#[tauri::command]
async fn get_compression_quality_options() -> Result<TauriResult<Vec<CompressionQualityInfo>>, String> {
    tracing::info!("📋 Getting compression quality options");

    let options = CompressionQuality::all()
        .into_iter()
        .map(|quality| CompressionQualityInfo {
            quality: quality.clone(),
            percentage: quality.percentage(),
            description: quality.description().to_string(),
            crf_value: quality.crf_value(),
            compression_ratio: quality.compression_ratio(),
        })
        .collect();

    Ok(TauriResult {
        data: options,
        message: Some("Compression quality options retrieved".to_string()),
    })
}

/// Estimate compressed file size (simple method for UI)
#[tauri::command]
async fn estimate_compression_size(
    original_size_mb: u64,
    quality: CompressionQuality,
) -> Result<TauriResult<CompressionEstimate>, String> {
    tracing::info!("📊 Estimating compression size for {} MB at {:?} quality", original_size_mb, quality);

    match CompressionService::new() {
        Ok(service) => {
            // Use simple estimation for now since we don't have the actual file path here
            // The real estimation will happen during actual compression
            let estimated_size = service.estimate_compressed_size_simple(original_size_mb, quality);
            let estimated_time = service.estimate_compression_time(original_size_mb, quality);
            let compression_ratio = quality.compression_ratio();

            let estimate = CompressionEstimate {
                original_size_mb,
                estimated_size_mb: estimated_size,
                compression_ratio,
                estimated_time_seconds: estimated_time,
                quality_used: quality,
                size_reduction_mb: original_size_mb.saturating_sub(estimated_size),
                size_reduction_percentage: ((1.0 - compression_ratio) * 100.0) as u8,
            };

            tracing::info!("✅ Estimated compressed size: {} MB ({}% reduction)",
                estimated_size, estimate.size_reduction_percentage);

            Ok(TauriResult {
                data: estimate,
                message: Some("Compression estimate calculated".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ Failed to estimate compression: {}", e);
            Err(format!("Compression estimation failed: {}", e))
        }
    }
}

/// Estimate compressed file size using actual file analysis
#[tauri::command]
async fn estimate_compression_size_from_file(
    file_path: String,
    quality: CompressionQuality,
) -> Result<TauriResult<CompressionEstimate>, String> {
    tracing::info!("📊 Analyzing file for compression estimation: {}", file_path);

    let input_path = std::path::Path::new(&file_path);
    if !input_path.exists() {
        return Err("File does not exist".to_string());
    }

    match CompressionService::new() {
        Ok(service) => {
            // Get original file size
            let original_size_mb = tokio::fs::metadata(input_path).await
                .map_err(|e| format!("Failed to read file metadata: {}", e))?
                .len() / 1024 / 1024;

            // Use accurate estimation based on file analysis
            let estimated_size = match service.estimate_compressed_size(input_path, quality).await {
                Ok(size) => size,
                Err(_) => service.estimate_compressed_size_simple(original_size_mb, quality),
            };

            let estimated_time = service.estimate_compression_time(original_size_mb, quality);
            let compression_ratio = estimated_size as f64 / original_size_mb as f64;

            let estimate = CompressionEstimate {
                original_size_mb,
                estimated_size_mb: estimated_size,
                compression_ratio,
                estimated_time_seconds: estimated_time,
                quality_used: quality,
                size_reduction_mb: original_size_mb.saturating_sub(estimated_size),
                size_reduction_percentage: ((1.0 - compression_ratio) * 100.0) as u8,
            };

            tracing::info!("✅ Accurate compression estimate: {} MB -> {} MB ({}% reduction)",
                original_size_mb, estimated_size, estimate.size_reduction_percentage);

            Ok(TauriResult {
                data: estimate,
                message: Some("Accurate compression size estimated successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ Failed to estimate compression size from file: {}", e);
            Err(format!("Failed to estimate compression size: {}", e))
        }
    }
}

/// Compress a video file
#[tauri::command]
async fn compress_video(
    input_path: String,
    output_path: String,
    quality: CompressionQuality,
    preserve_original: bool,
) -> Result<TauriResult<CompressionResult>, String> {
    tracing::info!("🎬 Starting video compression: {} -> {}", input_path, output_path);

    let service = CompressionService::new()
        .map_err(|e| format!("Failed to initialize compression service: {}", e))?;

    let options = CompressionOptions {
        quality,
        input_path: input_path.into(),
        output_path: output_path.into(),
        preserve_original,
        codec: "libx264".to_string(),
        preset: "medium".to_string(),
    };

    match service.compress_video(options, None).await {
        Ok(result) => {
            tracing::info!("✅ Compression completed successfully");
            Ok(TauriResult {
                data: result,
                message: Some("Video compression completed successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ Compression failed: {}", e);
            Err(format!("Video compression failed: {}", e))
        }
    }
}

/// Get compression progress
#[tauri::command]
async fn get_compression_progress(compression_id: String) -> Result<TauriResult<Option<CompressionProgress>>, String> {
    tracing::info!("📊 Getting compression progress for ID: {}", compression_id);

    let service = CompressionService::new()
        .map_err(|e| format!("Failed to initialize compression service: {}", e))?;

    let progress = service.get_compression_progress(&compression_id).await;

    Ok(TauriResult {
        data: progress,
        message: Some("Compression progress retrieved".to_string()),
    })
}

/// Cancel compression
#[tauri::command]
async fn cancel_compression(compression_id: String) -> Result<TauriResult<bool>, String> {
    tracing::info!("🛑 Cancelling compression: {}", compression_id);

    let service = CompressionService::new()
        .map_err(|e| format!("Failed to initialize compression service: {}", e))?;

    match service.cancel_compression(&compression_id).await {
        Ok(_) => {
            tracing::info!("✅ Compression cancelled successfully");
            Ok(TauriResult {
                data: true,
                message: Some("Compression cancelled successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ Failed to cancel compression: {}", e);
            Err(format!("Failed to cancel compression: {}", e))
        }
    }
}

// ============================================================================
// COMPRESSION HELPER TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionQualityInfo {
    pub quality: CompressionQuality,
    pub percentage: u8,
    pub description: String,
    pub crf_value: u8,
    pub compression_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionEstimate {
    pub original_size_mb: u64,
    pub estimated_size_mb: u64,
    pub compression_ratio: f64,
    pub estimated_time_seconds: u64,
    pub quality_used: CompressionQuality,
    pub size_reduction_mb: u64,
    pub size_reduction_percentage: u8,
}

// ============================================================================
// BATCH PROCESSING COMMANDS
// ============================================================================

use std::sync::OnceLock;

static BATCH_PROCESSOR: OnceLock<BatchProcessor> = OnceLock::new();

async fn get_batch_processor() -> Result<&'static BatchProcessor, String> {
    if let Some(processor) = BATCH_PROCESSOR.get() {
        Ok(processor)
    } else {
        match BatchProcessor::new().await {
            Ok(processor) => {
                match BATCH_PROCESSOR.set(processor) {
                    Ok(_) => Ok(BATCH_PROCESSOR.get().unwrap()),
                    Err(_) => Err("Failed to initialize batch processor".to_string()),
                }
            }
            Err(e) => Err(format!("Failed to create batch processor: {}", e)),
        }
    }
}

/// Start batch processing for multiple URLs
#[tauri::command]
async fn start_batch_processing(
    name: String,
    urls: Vec<String>,
    options: Option<BatchOptions>,
) -> Result<TauriResult<String>, String> {
    tracing::info!("🚀 Starting batch processing: '{}' with {} URLs", name, urls.len());

    let processor = get_batch_processor().await?;

    match processor.start_batch(name, urls, options).await {
        Ok(batch_id) => {
            tracing::info!("✅ Batch processing started with ID: {}", batch_id);
            Ok(TauriResult {
                data: batch_id,
                message: Some("Batch processing started successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ Failed to start batch processing: {}", e);
            Err(format!("Failed to start batch processing: {}", e))
        }
    }
}

/// Get progress for a specific batch
#[tauri::command]
async fn get_batch_progress(batch_id: String) -> Result<TauriResult<Option<BatchProgress>>, String> {
    let processor = get_batch_processor().await?;
    let progress = processor.get_batch_progress(&batch_id).await;

    // Log progress requests for debugging
    if let Some(ref progress_data) = progress {
        tracing::debug!("🔍 Frontend requested batch progress: {}% ({}/{} completed, {} active, {} failed)",
            progress_data.progress_percentage,
            progress_data.completed_items,
            progress_data.total_items,
            progress_data.active_items,
            progress_data.failed_items);
    } else {
        tracing::debug!("🔍 Frontend requested batch progress for {}: not found", batch_id);
    }

    Ok(TauriResult {
        data: progress,
        message: Some("Batch progress retrieved".to_string()),
    })
}

/// Get full batch information
#[tauri::command]
async fn get_batch_info(batch_id: String) -> Result<TauriResult<Option<BatchJob>>, String> {
    let processor = get_batch_processor().await?;
    let batch = processor.get_batch(&batch_id).await;

    // Log batch info requests for debugging
    if let Some(ref batch_data) = batch {
        tracing::debug!("🔍 Frontend requested batch info: {}% ({}/{} completed, {} active, {} failed)",
            batch_data.progress.progress_percentage,
            batch_data.progress.completed_items,
            batch_data.progress.total_items,
            batch_data.progress.active_items,
            batch_data.progress.failed_items);
    } else {
        tracing::debug!("🔍 Frontend requested batch info for {}: not found", batch_id);
    }

    Ok(TauriResult {
        data: batch,
        message: Some("Batch information retrieved".to_string()),
    })
}

/// Get all active batches
#[tauri::command]
async fn get_all_batches() -> Result<TauriResult<Vec<BatchJob>>, String> {
    let processor = get_batch_processor().await?;
    let batches = processor.get_all_batches().await;

    Ok(TauriResult {
        data: batches,
        message: Some("All batches retrieved".to_string()),
    })
}

/// Pause a batch
#[tauri::command]
async fn pause_batch(batch_id: String) -> Result<TauriResult<bool>, String> {
    tracing::info!("⏸️ Pausing batch: {}", batch_id);

    let processor = get_batch_processor().await?;

    match processor.pause_batch(&batch_id).await {
        Ok(_) => {
            tracing::info!("✅ Batch paused successfully");
            Ok(TauriResult {
                data: true,
                message: Some("Batch paused successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ Failed to pause batch: {}", e);
            Err(format!("Failed to pause batch: {}", e))
        }
    }
}

/// Resume a paused batch
#[tauri::command]
async fn resume_batch(batch_id: String) -> Result<TauriResult<bool>, String> {
    tracing::info!("▶️ Resuming batch: {}", batch_id);

    let processor = get_batch_processor().await?;

    match processor.resume_batch(&batch_id).await {
        Ok(_) => {
            tracing::info!("✅ Batch resumed successfully");
            Ok(TauriResult {
                data: true,
                message: Some("Batch resumed successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ Failed to resume batch: {}", e);
            Err(format!("Failed to resume batch: {}", e))
        }
    }
}

/// Cancel a batch
#[tauri::command]
async fn cancel_batch(batch_id: String) -> Result<TauriResult<bool>, String> {
    tracing::info!("🛑 Cancelling batch: {}", batch_id);

    let processor = get_batch_processor().await?;

    match processor.cancel_batch(&batch_id).await {
        Ok(_) => {
            tracing::info!("✅ Batch cancelled successfully");
            Ok(TauriResult {
                data: true,
                message: Some("Batch cancelled successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ Failed to cancel batch: {}", e);
            Err(format!("Failed to cancel batch: {}", e))
        }
    }
}

/// Remove a completed batch
#[tauri::command]
async fn remove_batch(batch_id: String) -> Result<TauriResult<bool>, String> {
    tracing::info!("🗑️ Removing batch: {}", batch_id);

    let processor = get_batch_processor().await?;

    match processor.remove_batch(&batch_id).await {
        Ok(_) => {
            tracing::info!("✅ Batch removed successfully");
            Ok(TauriResult {
                data: true,
                message: Some("Batch removed successfully".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("❌ Failed to remove batch: {}", e);
            Err(format!("Failed to remove batch: {}", e))
        }
    }
}

/// Clean up finished batches
#[tauri::command]
async fn cleanup_finished_batches() -> Result<TauriResult<bool>, String> {
    tracing::info!("🧹 Cleaning up finished batches");

    let processor = get_batch_processor().await?;
    processor.cleanup_finished_batches().await;

    tracing::info!("✅ Finished batches cleaned up");
    Ok(TauriResult {
        data: true,
        message: Some("Finished batches cleaned up successfully".to_string()),
    })
}

/// Open directory dialog for selecting download folder
#[tauri::command]
async fn open_directory_dialog(_app_handle: tauri::AppHandle) -> Result<TauriResult<Option<String>>, String> {
    tracing::info!("📁 Directory dialog not implemented yet");

    // TODO: Implement directory dialog when Tauri v2 API is stable
    Ok(TauriResult {
        data: None,
        message: Some("Directory dialog not implemented yet".to_string()),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            test_tauri_connection,
            get_app_version,
            read_clipboard,
            write_clipboard,
            validate_facebook_url,
            extract_video_info,
            fetch_thumbnail_image,
            download_video,
            check_compression_availability,
            get_compression_quality_options,
            estimate_compression_size,
            estimate_compression_size_from_file,
            compress_video,
            get_compression_progress,
            cancel_compression,
            start_batch_processing,
            get_batch_progress,
            get_batch_info,
            get_all_batches,
            pause_batch,
            resume_batch,
            cancel_batch,
            remove_batch,
            cleanup_finished_batches,
            open_directory_dialog
        ])
        .setup(|app| {
            // Initialize logging
            tracing_subscriber::fmt::init();
            tracing::info!("🚀 Facebook Video Downloader starting up...");

            // Ensure the main window is visible and focused
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                tracing::info!("✅ Main window shown and focused");
            } else {
                tracing::warn!("⚠️ Main window not found");
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
