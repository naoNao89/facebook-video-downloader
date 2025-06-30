//! # Video Compression Module
//!
//! Provides video compression functionality using FFmpeg with configurable quality settings.
//! Supports multiple compression levels with accurate file size estimation and progress tracking.

use crate::common::error::{FacebookExtractorError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;

/// Compression quality levels with corresponding CRF values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionQuality {
    /// 90% quality - CRF 18 (high quality, larger file)
    High,
    /// 50% quality - CRF 23 (balanced quality/size)
    Medium,
    /// 30% quality - CRF 28 (lower quality, smaller file)
    Low,
    /// 10% quality - CRF 35 (lowest quality, minimal file size)
    Minimal,
}

impl CompressionQuality {
    /// Get the CRF value for this quality level
    pub fn crf_value(&self) -> u8 {
        match self {
            CompressionQuality::High => 18,
            CompressionQuality::Medium => 23,
            CompressionQuality::Low => 28,
            CompressionQuality::Minimal => 35,
        }
    }

    /// Get the quality percentage
    pub fn percentage(&self) -> u8 {
        match self {
            CompressionQuality::High => 90,
            CompressionQuality::Medium => 50,
            CompressionQuality::Low => 30,
            CompressionQuality::Minimal => 10,
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            CompressionQuality::High => "High quality, smaller file",
            CompressionQuality::Medium => "Balanced quality/size",
            CompressionQuality::Low => "Lower quality, smaller file",
            CompressionQuality::Minimal => "Lowest quality, minimal file size",
        }
    }

    /// Estimate compression ratio compared to original
    pub fn compression_ratio(&self) -> f64 {
        match self {
            CompressionQuality::High => 0.85,    // 15% reduction
            CompressionQuality::Medium => 0.60,  // 40% reduction
            CompressionQuality::Low => 0.40,     // 60% reduction
            CompressionQuality::Minimal => 0.25, // 75% reduction
        }
    }

    /// Get all available quality options
    pub fn all() -> Vec<CompressionQuality> {
        vec![
            CompressionQuality::High,
            CompressionQuality::Medium,
            CompressionQuality::Low,
            CompressionQuality::Minimal,
        ]
    }

    /// Get a short label for this quality level
    pub fn label(&self) -> &'static str {
        match self {
            CompressionQuality::High => "high",
            CompressionQuality::Medium => "medium",
            CompressionQuality::Low => "low",
            CompressionQuality::Minimal => "minimal",
        }
    }
}

/// Compression options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionOptions {
    pub quality: CompressionQuality,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub preserve_original: bool,
    pub codec: String,
    pub preset: String,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            quality: CompressionQuality::Medium,
            input_path: PathBuf::new(),
            output_path: PathBuf::new(),
            preserve_original: true,
            codec: "libx264".to_string(),
            preset: "medium".to_string(),
        }
    }
}

/// Compression progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionProgress {
    pub id: String,
    pub status: CompressionStatus,
    pub progress_percentage: f64,
    pub current_frame: Option<u64>,
    pub total_frames: Option<u64>,
    pub fps: Option<f64>,
    pub speed: Option<String>,
    pub eta_seconds: Option<u64>,
    pub input_size_mb: u64,
    pub estimated_output_size_mb: u64,
    pub actual_output_size_mb: Option<u64>,
    pub compression_ratio: Option<f64>,
    pub error_message: Option<String>,
}

/// Compression status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionStatus {
    Queued,
    Analyzing,
    Compressing,
    Completed,
    Failed,
    Cancelled,
}

/// Video probe metadata for accurate size estimation
#[derive(Debug, Clone)]
struct VideoProbeMetadata {
    pub duration_seconds: u32,
    pub bitrate_bps: u64,
    pub width: u32,
    pub height: u32,
}

/// Compression result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub id: String,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub original_size_mb: u64,
    pub compressed_size_mb: u64,
    pub compression_ratio: f64,
    pub quality_used: CompressionQuality,
    pub processing_time_seconds: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Video compression service
pub struct CompressionService {
    ffmpeg_path: Option<PathBuf>,
    active_compressions: Arc<Mutex<Vec<CompressionProgress>>>,
}

impl CompressionService {
    /// Create a new compression service
    pub fn new() -> Result<Self> {
        let ffmpeg_path = Self::find_ffmpeg_path();

        if ffmpeg_path.is_none() {
            return Err(FacebookExtractorError::compression(
                "FFmpeg not found. Please install FFmpeg to use compression features."
            ));
        }

        Ok(Self {
            ffmpeg_path,
            active_compressions: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Create a compression service without FFmpeg (compression will be disabled)
    pub fn new_disabled() -> Self {
        Self {
            ffmpeg_path: None,
            active_compressions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Find FFmpeg executable path
    fn find_ffmpeg_path() -> Option<PathBuf> {
        // Try common locations and PATH
        let candidates = vec![
            "ffmpeg",
            "ffmpeg.exe",
            "/usr/bin/ffmpeg",
            "/usr/local/bin/ffmpeg",
            "/opt/homebrew/bin/ffmpeg",
            "C:\\ffmpeg\\bin\\ffmpeg.exe",
        ];

        for candidate in candidates {
            if let Ok(output) = Command::new(candidate)
                .arg("-version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()
            {
                if output.status.success() {
                    return Some(PathBuf::from(candidate));
                }
            }
        }

        None
    }

    /// Check if FFmpeg is available
    pub fn is_available(&self) -> bool {
        self.ffmpeg_path.is_some()
    }

    /// Get FFmpeg version information
    pub async fn get_ffmpeg_version(&self) -> Result<String> {
        let ffmpeg_path = self.ffmpeg_path.as_ref()
            .ok_or_else(|| FacebookExtractorError::compression("FFmpeg not available"))?;

        let output = Command::new(ffmpeg_path)
            .arg("-version")
            .output()
            .map_err(|e| FacebookExtractorError::compression(format!("Failed to run FFmpeg: {}", e)))?;

        if !output.status.success() {
            return Err(FacebookExtractorError::compression("Failed to get FFmpeg version"));
        }

        let version_output = String::from_utf8_lossy(&output.stdout);
        let first_line = version_output.lines().next().unwrap_or("Unknown version");
        
        Ok(first_line.to_string())
    }

    /// Estimate compressed file size using FFmpeg probe for accurate analysis
    pub async fn estimate_compressed_size(&self, input_path: &std::path::Path, quality: CompressionQuality) -> Result<u64> {
        // Try to get accurate video metadata using FFmpeg probe
        match self.probe_video_metadata(input_path).await {
            Ok(metadata) => {
                // Calculate estimated size based on actual video properties
                let estimated_bitrate = self.calculate_target_bitrate(&metadata, quality);
                let estimated_size_mb = (estimated_bitrate * metadata.duration_seconds as u64) / 8 / 1024 / 1024;
                Ok(estimated_size_mb.max(1)) // Minimum 1 MB
            }
            Err(_) => {
                // Fallback to file size based estimation if probe fails
                let file_size = tokio::fs::metadata(input_path).await
                    .map_err(|e| FacebookExtractorError::compression(format!("Failed to read file size: {}", e)))?
                    .len() / 1024 / 1024;
                let ratio = quality.compression_ratio();
                Ok(((file_size as f64) * ratio).round() as u64)
            }
        }
    }

    /// Legacy method for backward compatibility
    pub fn estimate_compressed_size_simple(&self, original_size_mb: u64, quality: CompressionQuality) -> u64 {
        let ratio = quality.compression_ratio();
        ((original_size_mb as f64) * ratio).round() as u64
    }

    /// Estimate compression time based on file size and quality
    pub fn estimate_compression_time(&self, size_mb: u64, quality: CompressionQuality) -> u64 {
        // Base time estimation: ~2 seconds per MB for medium quality
        let base_time_per_mb = match quality {
            CompressionQuality::High => 3.0,     // Slower for higher quality
            CompressionQuality::Medium => 2.0,   // Baseline
            CompressionQuality::Low => 1.5,      // Faster for lower quality
            CompressionQuality::Minimal => 1.0,  // Fastest
        };

        ((size_mb as f64) * base_time_per_mb).round() as u64
    }
}

impl Default for CompressionService {
    fn default() -> Self {
        Self::new_disabled()
    }
}

impl CompressionService {
    /// Probe video metadata using FFmpeg
    async fn probe_video_metadata(&self, input_path: &std::path::Path) -> Result<VideoProbeMetadata> {
        let ffmpeg_path = self.ffmpeg_path.as_ref()
            .ok_or_else(|| FacebookExtractorError::compression("FFmpeg not available"))?;

        // Use ffprobe to get video metadata
        let output = tokio::task::spawn_blocking({
            let ffmpeg_path = ffmpeg_path.clone();
            let input_path = input_path.to_path_buf();
            move || {
                Command::new(ffmpeg_path.parent().unwrap_or(&ffmpeg_path).join("ffprobe"))
                    .arg("-v").arg("quiet")
                    .arg("-print_format").arg("json")
                    .arg("-show_format")
                    .arg("-show_streams")
                    .arg(&input_path)
                    .output()
            }
        }).await
        .map_err(|e| FacebookExtractorError::compression(format!("Failed to spawn ffprobe task: {}", e)))?
        .map_err(|e| FacebookExtractorError::compression(format!("Failed to execute ffprobe: {}", e)))?;

        if !output.status.success() {
            return Err(FacebookExtractorError::compression("Failed to probe video metadata"));
        }

        let probe_output = String::from_utf8_lossy(&output.stdout);
        self.parse_probe_output(&probe_output)
    }

    /// Parse FFmpeg probe output to extract video metadata
    fn parse_probe_output(&self, json_output: &str) -> Result<VideoProbeMetadata> {
        // Simple JSON parsing for the fields we need
        let duration = self.extract_json_field(json_output, "duration")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(60.0); // Default 1 minute

        let bitrate = self.extract_json_field(json_output, "bit_rate")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(2000000); // Default 2 Mbps

        let width = self.extract_json_field(json_output, "width")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1280);

        let height = self.extract_json_field(json_output, "height")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(720);

        Ok(VideoProbeMetadata {
            duration_seconds: duration as u32,
            bitrate_bps: bitrate,
            width,
            height,
        })
    }

    /// Extract a field value from JSON output (simple string parsing)
    fn extract_json_field(&self, json: &str, field: &str) -> Option<String> {
        let pattern = format!("\"{}\":", field);
        if let Some(start) = json.find(&pattern) {
            let start = start + pattern.len();
            let remaining = &json[start..];

            // Skip whitespace and quotes
            let remaining = remaining.trim_start().trim_start_matches('"');

            // Find the end of the value
            if let Some(end) = remaining.find(&['"', ',', '}', '\n'][..]) {
                return Some(remaining[..end].trim().to_string());
            }
        }
        None
    }

    /// Calculate target bitrate for compression quality
    fn calculate_target_bitrate(&self, metadata: &VideoProbeMetadata, quality: CompressionQuality) -> u64 {
        // Base target bitrates for different resolutions and quality levels
        let base_bitrate = match (metadata.width, metadata.height) {
            (w, h) if w >= 1920 || h >= 1080 => 8000000,  // 1080p: 8 Mbps
            (w, h) if w >= 1280 || h >= 720 => 5000000,   // 720p: 5 Mbps
            (w, h) if w >= 854 || h >= 480 => 2500000,    // 480p: 2.5 Mbps
            _ => 1500000,                                  // 360p: 1.5 Mbps
        };

        // Apply quality multiplier
        let quality_multiplier = match quality {
            CompressionQuality::High => 0.85,     // 85% of base bitrate
            CompressionQuality::Medium => 0.60,   // 60% of base bitrate
            CompressionQuality::Low => 0.40,      // 40% of base bitrate
            CompressionQuality::Minimal => 0.25,  // 25% of base bitrate
        };

        ((base_bitrate as f64) * quality_multiplier) as u64
    }

    /// Compress a video file with the specified options
    pub async fn compress_video(
        &self,
        options: CompressionOptions,
        progress_callback: Option<Box<dyn Fn(CompressionProgress) + Send + Sync>>,
    ) -> Result<CompressionResult> {
        let ffmpeg_path = self.ffmpeg_path.as_ref()
            .ok_or_else(|| FacebookExtractorError::compression("FFmpeg not available"))?;

        let compression_id = uuid::Uuid::new_v4().to_string();
        let start_time = Instant::now();

        // Get input file size
        let input_metadata = tokio::fs::metadata(&options.input_path).await
            .map_err(|e| FacebookExtractorError::compression(format!("Failed to read input file: {}", e)))?;
        let input_size_mb = input_metadata.len() / 1024 / 1024;

        // Estimate output size using probe if possible
        let estimated_output_size_mb = match self.estimate_compressed_size(&options.input_path, options.quality).await {
            Ok(size) => size,
            Err(_) => self.estimate_compressed_size_simple(input_size_mb, options.quality),
        };

        // Initialize progress
        let mut progress = CompressionProgress {
            id: compression_id.clone(),
            status: CompressionStatus::Analyzing,
            progress_percentage: 0.0,
            current_frame: None,
            total_frames: None,
            fps: None,
            speed: None,
            eta_seconds: None,
            input_size_mb,
            estimated_output_size_mb,
            actual_output_size_mb: None,
            compression_ratio: None,
            error_message: None,
        };

        // Add to active compressions
        {
            let mut active = self.active_compressions.lock().await;
            active.push(progress.clone());
        }

        // Notify progress callback
        if let Some(ref callback) = progress_callback {
            callback(progress.clone());
        }

        // Build FFmpeg command with proper audio preservation
        let mut cmd = Command::new(ffmpeg_path);
        cmd.arg("-i")
           .arg(&options.input_path)
           .arg("-c:v")
           .arg(&options.codec)
           .arg("-crf")
           .arg(options.quality.crf_value().to_string())
           .arg("-preset")
           .arg(&options.preset)
           // Copy audio stream to preserve original quality and avoid re-encoding
           .arg("-c:a")
           .arg("copy")
           // Ensure all audio streams are preserved
           .arg("-map")
           .arg("0:v:0") // Map first video stream
           .arg("-map")
           .arg("0:a?") // Map all audio streams (? makes it optional)
           // Optimize for web playback
           .arg("-movflags")
           .arg("+faststart")
           // Overwrite output file without prompting
           .arg("-y")
           .arg(&options.output_path);

        // Update progress to compressing
        progress.status = CompressionStatus::Compressing;
        progress.progress_percentage = 5.0;
        if let Some(ref callback) = progress_callback {
            callback(progress.clone());
        }

        // Execute compression
        let output = tokio::task::spawn_blocking(move || cmd.output()).await
            .map_err(|e| FacebookExtractorError::compression(format!("Failed to spawn FFmpeg task: {}", e)))?
            .map_err(|e| FacebookExtractorError::compression(format!("Failed to execute FFmpeg: {}", e)))?;

        let processing_time = start_time.elapsed().as_secs();

        // Check if compression was successful
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            progress.status = CompressionStatus::Failed;
            progress.error_message = Some(error_msg.to_string());

            if let Some(ref callback) = progress_callback {
                callback(progress.clone());
            }

            return Err(FacebookExtractorError::compression(format!(
                "FFmpeg compression failed: {}", error_msg
            )));
        }

        // Get output file size
        let output_metadata = tokio::fs::metadata(&options.output_path).await
            .map_err(|e| FacebookExtractorError::compression(format!("Failed to read output file: {}", e)))?;
        let output_size_mb = output_metadata.len() / 1024 / 1024;

        let compression_ratio = if input_size_mb > 0 {
            output_size_mb as f64 / input_size_mb as f64
        } else {
            1.0
        };

        // Update final progress
        progress.status = CompressionStatus::Completed;
        progress.progress_percentage = 100.0;
        progress.actual_output_size_mb = Some(output_size_mb);
        progress.compression_ratio = Some(compression_ratio);

        if let Some(ref callback) = progress_callback {
            callback(progress.clone());
        }

        // Clean up original file if requested
        if !options.preserve_original {
            if let Err(e) = tokio::fs::remove_file(&options.input_path).await {
                tracing::warn!("Failed to remove original file after compression: {}", e);
                // Don't fail the entire operation if cleanup fails
            } else {
                tracing::info!("✅ Original file cleaned up: {}", options.input_path.display());
            }
        }

        // Remove from active compressions
        {
            let mut active = self.active_compressions.lock().await;
            active.retain(|p| p.id != compression_id);
        }

        Ok(CompressionResult {
            id: compression_id,
            input_path: options.input_path,
            output_path: options.output_path,
            original_size_mb: input_size_mb,
            compressed_size_mb: output_size_mb,
            compression_ratio,
            quality_used: options.quality,
            processing_time_seconds: processing_time,
            success: true,
            error_message: None,
        })
    }

    /// Get active compression progress
    pub async fn get_compression_progress(&self, id: &str) -> Option<CompressionProgress> {
        let active = self.active_compressions.lock().await;
        active.iter().find(|p| p.id == id).cloned()
    }

    /// Get all active compressions
    pub async fn get_active_compressions(&self) -> Vec<CompressionProgress> {
        let active = self.active_compressions.lock().await;
        active.clone()
    }

    /// Cancel a compression operation
    pub async fn cancel_compression(&self, id: &str) -> Result<()> {
        let mut active = self.active_compressions.lock().await;
        if let Some(progress) = active.iter_mut().find(|p| p.id == id) {
            progress.status = CompressionStatus::Cancelled;
            Ok(())
        } else {
            Err(FacebookExtractorError::compression(format!(
                "Compression not found: {}", id
            )))
        }
    }
}
