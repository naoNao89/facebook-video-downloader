//! Stream analysis and processing functionality

use crate::common::types::{VideoQuality, StreamType};
use crate::processing::file_size::{AccurateFileSizeService, bytes_to_mb};
use base64::{Engine as _, engine::general_purpose};
use tracing::{debug, info, warn};

/// Stream analyzer for Facebook videos
pub struct StreamAnalyzer;

impl StreamAnalyzer {
    /// Create a new stream analyzer
    pub fn new() -> Self {
        Self
    }

    /// Comprehensive analysis of Facebook video stream
    pub fn analyze_facebook_video_stream(&self, url: &str) -> VideoQuality {
        self.analyze_facebook_video_stream_with_duration(url, None)
    }

    /// Comprehensive analysis of Facebook video stream with optional duration for accurate file size estimation
    pub fn analyze_facebook_video_stream_with_duration(&self, url: &str, duration_seconds: Option<u32>) -> VideoQuality {
        // Extract efg metadata from URL
        let efg_metadata = self.extract_efg_metadata(url);

        // Determine stream type from efg metadata
        let mut stream_type = self.determine_stream_type(&efg_metadata);

        // If EFG analysis failed, use URL pattern fallback
        if stream_type == StreamType::Unknown {
            stream_type = self.determine_stream_type_from_url(url);
        }

        // Extract quality information
        let (quality_name, width, height) = if !efg_metadata.is_empty() {
            self.extract_quality_from_efg(&efg_metadata)
        } else {
            self.extract_quality_from_url(url)
        };

        // Use actual duration if provided, otherwise fall back to default
        let duration = duration_seconds.unwrap_or(120); // Default 2 minutes (more realistic for Facebook)

        // Estimate file size based on quality and actual duration
        let estimated_size_mb = self.estimate_file_size(width, height, duration);

        VideoQuality {
            quality: quality_name,
            size: format!("~{} MB", estimated_size_mb),
            format: "MP4".to_string(),
            download_url: url.to_string(),
            width,
            height,
            stream_type,
            efg_metadata,
            estimated_size_mb,
            bitrate_kbps: None,
            fps: None,
            codec: None,
        }
    }

    /// Analyze Facebook video stream with accurate file size detection
    pub async fn analyze_facebook_video_stream_with_accurate_size(&self, url: &str, duration_seconds: Option<u32>) -> VideoQuality {
        // Get basic stream analysis first
        let mut quality = self.analyze_facebook_video_stream_with_duration(url, duration_seconds);

        // Try to get accurate file size using the new service
        match AccurateFileSizeService::new() {
            Ok(size_service) => {
                match size_service.get_accurate_file_size(url).await {
                    Ok(actual_size_bytes) => {
                        let actual_size_mb = bytes_to_mb(actual_size_bytes);
                        info!("✅ Got accurate file size: {} MB (was estimated: {} MB)",
                              actual_size_mb, quality.estimated_size_mb);

                        // Update the quality with accurate size
                        quality.estimated_size_mb = actual_size_mb;
                        quality.size = format!("{} MB", actual_size_mb);
                    }
                    Err(e) => {
                        warn!("Failed to get accurate file size for {}: {}", url, e);
                        debug!("Falling back to estimated size: {} MB", quality.estimated_size_mb);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create AccurateFileSizeService: {}", e);
            }
        }

        quality
    }

    /// Extract efg metadata from Facebook video URL
    fn extract_efg_metadata(&self, url: &str) -> String {
        if let Some(efg_start) = url.find("efg=") {
            let efg_part = &url[efg_start + 4..];
            let encoded_efg = if let Some(efg_end) = efg_part.find('&') {
                &efg_part[..efg_end]
            } else {
                efg_part
            };

            // Enhanced URL decoding with multiple attempts to handle double encoding
            let mut url_decoded = encoded_efg.to_string();

            // Handle special case of u00253D (double-encoded %3D)
            if url_decoded.contains("u00253D") {
                url_decoded = url_decoded.replace("u00253D", "%3D");
            }

            // Handle other double-encoded patterns
            if url_decoded.contains("u0025") {
                url_decoded = url_decoded.replace("u0025", "%");
            }

            // First URL decode attempt using manual decoding
            let decoded = self.url_decode(&url_decoded);
            url_decoded = decoded;

            // Check if it's still URL-encoded (double encoding fix)
            if url_decoded.contains("%") {
                let double_decoded = self.url_decode(&url_decoded);
                url_decoded = double_decoded;
            }

            // Try base64 decode to JSON
            match general_purpose::STANDARD.decode(url_decoded.as_bytes()) {
                Ok(base64_decoded) => {
                    match String::from_utf8(base64_decoded) {
                        Ok(json_string) => return json_string,
                        Err(_) => {}
                    }
                }
                Err(_) => {
                    // Try adding base64 padding if missing
                    let mut padded = url_decoded.clone();
                    while padded.len() % 4 != 0 {
                        padded.push('=');
                    }

                    match general_purpose::STANDARD.decode(padded.as_bytes()) {
                        Ok(base64_decoded) => {
                            if let Ok(json_string) = String::from_utf8(base64_decoded) {
                                return json_string;
                            }
                        }
                        Err(_) => {}
                    }
                }
            }

            // Return URL-decoded version for quality extraction
            return url_decoded;
        }
        String::new()
    }

    /// Manual URL decoding to replace urlencoding dependency
    fn url_decode(&self, input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                // Try to decode the next two characters as hex
                let hex1 = chars.next();
                let hex2 = chars.next();

                if let (Some(h1), Some(h2)) = (hex1, hex2) {
                    let hex_str = format!("{}{}", h1, h2);
                    if let Ok(byte_val) = u8::from_str_radix(&hex_str, 16) {
                        result.push(byte_val as char);
                    } else {
                        // If hex parsing fails, keep the original characters
                        result.push('%');
                        result.push(h1);
                        result.push(h2);
                    }
                } else {
                    // If we don't have two more characters, keep the %
                    result.push('%');
                    if let Some(h1) = hex1 {
                        result.push(h1);
                    }
                    if let Some(h2) = hex2 {
                        result.push(h2);
                    }
                }
            } else if ch == '+' {
                // Replace + with space (common in URL encoding)
                result.push(' ');
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Determine stream type from efg metadata using streamlined analysis
    fn determine_stream_type(&self, efg_metadata: &str) -> StreamType {
        if efg_metadata.is_empty() {
            return StreamType::Unknown;
        }

        // Try JSON parsing first (for successfully decoded metadata)
        if let Some(stream_type) = self.parse_efg_json(efg_metadata) {
            return stream_type;
        }

        // Try base64 decoding for URL-encoded metadata
        if efg_metadata.starts_with("eyJ") {
            if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(efg_metadata.as_bytes()) {
                if let Ok(json_str) = String::from_utf8(decoded_bytes) {
                    if let Some(stream_type) = self.parse_efg_json(&json_str) {
                        return stream_type;
                    }
                }
            }
        }

        // Enhanced fallback using quality and codec indicators
        if efg_metadata.contains("dash_ln_heaac")
            || efg_metadata.contains("heaac_vbr3_audio")
            || efg_metadata.contains("dash_ln_heaac_vbr3_audio")
            || efg_metadata.contains("_audio")
        {
            StreamType::AudioOnly
        } else if efg_metadata.contains("progressive") || efg_metadata.contains("xpv_progressive") {
            StreamType::CompleteVideoAudio
        } else if efg_metadata.contains("dash_vp9") || efg_metadata.contains("dash_h264") {
            if efg_metadata.contains("720p")
                || efg_metadata.contains("540p")
                || efg_metadata.contains("480p")
                || efg_metadata.contains("1080p")
                || efg_metadata.contains("360p")
                || efg_metadata.contains("240p")
            {
                StreamType::VideoOnly
            } else {
                StreamType::VideoOnly
            }
        } else if efg_metadata.contains("720p")
            || efg_metadata.contains("540p")
            || efg_metadata.contains("480p")
            || efg_metadata.contains("1080p")
            || efg_metadata.contains("360p")
            || efg_metadata.contains("240p")
        {
            // If we have quality indicators but no explicit audio/video-only markers,
            // assume it's a complete stream (this is often the case for Facebook videos)
            StreamType::CompleteVideoAudio
        } else {
            StreamType::Unknown
        }
    }

    /// Parse EFG JSON content to determine stream type
    fn parse_efg_json(&self, json_content: &str) -> Option<StreamType> {
        // Try JSON parsing first
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(json_content) {
            if let Some(vencode_tag) = json_value.get("vencode_tag").and_then(|v| v.as_str()) {
                if vencode_tag.contains("xpv_progressive") {
                    return Some(StreamType::CompleteVideoAudio);
                } else if vencode_tag.contains("dash_ln_heaac") || vencode_tag.ends_with("_audio") {
                    return Some(StreamType::AudioOnly);
                } else if vencode_tag.contains("dash_vp9") || vencode_tag.contains("dash_h264") {
                    if vencode_tag.contains("_audio") {
                        return Some(StreamType::AudioOnly);
                    } else {
                        return Some(StreamType::VideoOnly);
                    }
                }
            }
        } else {
            // Fallback to string pattern matching
            if json_content.contains("xpv_progressive") {
                return Some(StreamType::CompleteVideoAudio);
            } else if json_content.contains("dash_ln_heaac") || json_content.contains("_audio") {
                return Some(StreamType::AudioOnly);
            } else if json_content.contains("dash_vp9") || json_content.contains("dash_h264") {
                return Some(StreamType::VideoOnly);
            }
        }

        None
    }

    /// Fallback stream type detection using URL patterns
    fn determine_stream_type_from_url(&self, url: &str) -> StreamType {
        // Check for explicit complete video indicators
        if url.contains("browser_native_hd_url") || url.contains("browser_native_sd_url") {
            return StreamType::CompleteVideoAudio;
        }

        // Check for mobile format (usually complete)
        if url.contains("/m69/") {
            return StreamType::CompleteVideoAudio;
        }

        // Check for audio-only indicators
        if url.contains("dash_ln_heaac") || url.contains("_audio") {
            return StreamType::AudioOnly;
        }

        // If it's an MP4 without audio indicators, likely complete
        if url.contains(".mp4") && !url.contains("dash_ln_heaac") && !url.contains("_audio") {
            // Additional check: if URL has quality indicators, it's likely complete
            if url.contains("720") || url.contains("1080") || url.contains("480") ||
               url.contains("540") || url.contains("360") || url.contains("240") {
                return StreamType::CompleteVideoAudio;
            }
        }

        StreamType::Unknown
    }

    /// Extract quality information from efg metadata
    fn extract_quality_from_efg(&self, efg_metadata: &str) -> (String, u32, u32) {
        // Check for audio streams first
        if efg_metadata.contains("dash_ln_heaac")
            || efg_metadata.contains("heaac_vbr3_audio")
            || efg_metadata.contains("dash_ln_heaac_vbr3_audio")
            || efg_metadata.contains("_audio")
        {
            return ("Audio Stream".to_string(), 0, 0);
        }

        // Check for Facebook's quality level system (q30, q40, q50, etc.)
        if let Some(quality) = self.extract_quality_from_dash_levels(efg_metadata) {
            return quality;
        }

        // Check quality indicators in order of preference
        if efg_metadata.contains("1080p") || efg_metadata.contains("_1080p") {
            ("1080p Full HD".to_string(), 1920, 1080)
        } else if efg_metadata.contains("840p") || efg_metadata.contains("_840p") {
            ("840p".to_string(), 1488, 840)
        } else if efg_metadata.contains("720p") || efg_metadata.contains("_720p") {
            ("720p HD".to_string(), 1280, 720)
        } else if efg_metadata.contains("540p") || efg_metadata.contains("_540p") {
            ("540p".to_string(), 960, 540)
        } else if efg_metadata.contains("480p") || efg_metadata.contains("_480p") {
            ("480p SD".to_string(), 854, 480)
        } else if efg_metadata.contains("360p") || efg_metadata.contains("_360p") {
            ("360p".to_string(), 640, 360)
        } else if efg_metadata.contains("240p") || efg_metadata.contains("_240p") {
            ("240p".to_string(), 426, 240)
        } else {
            ("Original".to_string(), 0, 0)
        }
    }

    /// Extract quality from Facebook's DASH quality level system (q30, q40, q50, etc.)
    fn extract_quality_from_dash_levels(&self, efg_metadata: &str) -> Option<(String, u32, u32)> {
        // Facebook's DASH quality level mapping
        let quality_mappings = vec![
            ("_q90", "1080p Full HD", 1920, 1080),
            ("_q85", "1080p Full HD", 1920, 1080),
            ("_q80", "900p", 1600, 900),
            ("_q75", "840p", 1488, 840),
            ("_q70", "840p", 1488, 840),
            ("_q65", "720p HD", 1280, 720),
            ("_q60", "720p HD", 1280, 720),
            ("_q55", "720p HD", 1280, 720),
            ("_q50", "720p HD", 1280, 720),
            ("_q45", "540p", 960, 540),
            ("_q40", "540p", 960, 540),
            ("_q35", "480p SD", 854, 480),
            ("_q30", "480p SD", 854, 480),
            ("_q25", "360p", 640, 360),
            ("_q20", "360p", 640, 360),
            ("_q15", "240p", 426, 240),
            ("_q10", "240p", 426, 240),
        ];

        for (pattern, quality_name, width, height) in quality_mappings {
            if efg_metadata.contains(pattern) {
                return Some((quality_name.to_string(), width, height));
            }
        }

        None
    }

    /// Extract quality information from URL patterns (fallback)
    fn extract_quality_from_url(&self, url: &str) -> (String, u32, u32) {
        if url.contains("browser_native_hd_url") {
            ("720p HD (Native)".to_string(), 1280, 720)
        } else if url.contains("browser_native_sd_url") {
            ("360p SD (Native)".to_string(), 640, 360)
        } else if url.contains("1080") {
            ("1080p Full HD".to_string(), 1920, 1080)
        } else if url.contains("840") {
            ("840p".to_string(), 1488, 840)
        } else if url.contains("720") {
            ("720p HD".to_string(), 1280, 720)
        } else if url.contains("540") {
            ("540p".to_string(), 960, 540)
        } else if url.contains("480") {
            ("480p SD".to_string(), 854, 480)
        } else if url.contains("360") {
            ("360p".to_string(), 640, 360)
        } else {
            ("Original Quality".to_string(), 1280, 720)
        }
    }

    /// Estimate file size based on resolution and duration
    fn estimate_file_size(&self, width: u32, height: u32, duration_seconds: u32) -> u32 {
        // Special handling for audio streams (width=0, height=0)
        if width == 0 && height == 0 {
            let audio_bitrate = 128; // kbps
            let size_mb = (audio_bitrate * duration_seconds) / 8 / 1024;
            return size_mb.max(1).min(15);
        }

        // Realistic bitrate estimates (kbps) for Facebook videos based on resolution
        // Facebook uses aggressive compression, so these are lower than typical video bitrates
        let video_bitrate = match (width, height) {
            (1920, 1080) => 2500, // 1080p: ~2.5 Mbps (Facebook compressed)
            (1488, 840) => 2000,  // 840p: ~2 Mbps
            (1280, 720) => 1500,  // 720p: ~1.5 Mbps
            (960, 540) => 1000,   // 540p: ~1 Mbps
            (854, 480) => 800,    // 480p: ~0.8 Mbps
            (640, 360) => 600,    // 360p: ~0.6 Mbps
            _ => 1000,            // Default: ~1 Mbps
        };

        let audio_bitrate = 128; // ~128 kbps for audio
        let total_bitrate = video_bitrate + audio_bitrate;

        // Calculate file size: (bitrate in kbps * duration in seconds) / 8 / 1024
        // Convert from kilobits to kilobytes (/8), then to megabytes (/1024)
        let size_mb = (total_bitrate * duration_seconds) / 8 / 1024;
        size_mb.max(1) // Minimum 1 MB
    }

    /// Enhanced filtering for complete video+audio streams with improved detection
    /// This version includes better heuristics to identify complete streams that may be
    /// misclassified, while still excluding DASH combined streams.
    pub fn filter_complete_video_streams(&self, all_streams: Vec<VideoQuality>) -> Vec<VideoQuality> {
        println!("\n🔍 Enhanced filtering for complete video+audio content...");
        println!("📊 Input: {} total streams", all_streams.len());

        let mut progressive_streams = Vec::new();
        let mut potential_complete_streams = Vec::new();

        for (i, stream) in all_streams.iter().enumerate() {
            println!(
                "📊 Analyzing stream {}: {} - {:?} - {} MB - {}x{}",
                i + 1,
                stream.quality,
                stream.stream_type,
                stream.estimated_size_mb,
                stream.width,
                stream.height
            );

            // Skip COMBINED: URLs (DASH streams that require merging)
            if stream.download_url.starts_with("COMBINED:") {
                println!("   ❌ Skipped: COMBINED DASH stream");
                continue;
            }

            match stream.stream_type {
                StreamType::CompleteVideoAudio => {
                    // Include streams that are large enough to be real video files
                    // Be more lenient with size requirements when file size detection may have failed
                    if stream.estimated_size_mb == 1 {
                        // If size is exactly 1 MB, it's likely a fallback estimate due to failed size detection
                        // In this case, accept the stream if it meets any of these criteria:
                        // 1. Has valid video dimensions
                        // 2. URL suggests it's a valid video stream (contains .mp4 and video patterns)
                        // 3. EFG metadata suggests it's a progressive stream
                        let has_dimensions = stream.width > 0 && stream.height > 0;
                        let url_suggests_video = stream.download_url.contains(".mp4") &&
                                               (stream.download_url.contains("/m69/") ||
                                                stream.download_url.contains("progressive") ||
                                                !stream.download_url.contains("dash_ln_heaac"));
                        let efg_suggests_progressive = stream.efg_metadata.contains("xpv_progressive") ||
                                                     stream.efg_metadata.contains("progressive") ||
                                                     (!stream.efg_metadata.contains("dash_ln_heaac") &&
                                                      !stream.efg_metadata.is_empty());

                        if has_dimensions || url_suggests_video || efg_suggests_progressive {
                            println!("   ✅ Added: Complete video+audio stream (accepting despite small size - likely failed size detection)");
                            println!("      📐 Has dimensions: {} ({}x{})", has_dimensions, stream.width, stream.height);
                            println!("      🌐 URL suggests video: {}", url_suggests_video);
                            println!("      📄 EFG suggests progressive: {}", efg_suggests_progressive);
                            progressive_streams.push(stream.clone());
                            continue;
                        }
                    }

                    // Normal size threshold check
                    if stream.estimated_size_mb >= 5 {
                        println!("   ✅ Added: Complete video+audio stream");
                        progressive_streams.push(stream.clone());
                    } else {
                        println!("   ❌ Skipped: Too small ({} MB) and doesn't meet fallback criteria", stream.estimated_size_mb);
                    }
                }
                StreamType::Unknown => {
                    // For Unknown streams, use heuristics to determine if they're complete
                    if self.is_likely_complete_stream(stream) {
                        println!("   ✅ Added: Unknown stream identified as likely complete");
                        potential_complete_streams.push(stream.clone());
                    } else {
                        println!("   ❌ Skipped: Unknown stream doesn't meet complete criteria");
                    }
                }
                StreamType::VideoOnly => {
                    // Check if this "VideoOnly" stream is actually a complete stream
                    // that was misclassified due to conservative detection
                    if self.is_likely_complete_stream_despite_classification(stream) {
                        println!("   ✅ Added: VideoOnly stream identified as likely complete (reclassified)");
                        potential_complete_streams.push(stream.clone());
                    } else {
                        println!("   ❌ Skipped: VideoOnly stream type");
                    }
                }
                _ => {
                    println!("   ❌ Skipped: {:?} stream type", stream.stream_type);
                }
            }
        }

        // Combine confirmed and potential complete streams
        progressive_streams.extend(potential_complete_streams);

        println!("📊 After filtering: {} streams", progressive_streams.len());

        // Remove duplicates and sort by quality
        let final_streams = self.deduplicate_and_sort_streams(progressive_streams);

        println!("📊 After deduplication: {} final streams", final_streams.len());
        for (i, stream) in final_streams.iter().enumerate() {
            println!("   {}. {} ({}x{}) - {} MB",
                i + 1, stream.quality, stream.width, stream.height, stream.estimated_size_mb);
        }

        final_streams
    }

    /// Heuristic to determine if an Unknown stream is likely a complete video+audio stream
    fn is_likely_complete_stream(&self, stream: &VideoQuality) -> bool {
        // Check if it has reasonable video dimensions
        let has_video_dimensions = stream.width > 0 && stream.height > 0;

        // Check if it's large enough to contain both video and audio
        // Be more lenient if size detection likely failed (exactly 1 MB suggests fallback estimate)
        let reasonable_size = if stream.estimated_size_mb == 1 && has_video_dimensions {
            // Accept 1 MB streams if they have valid dimensions (likely failed size detection)
            true
        } else {
            stream.estimated_size_mb >= 5
        };

        // Check URL patterns that suggest complete streams
        let url_suggests_complete = stream.download_url.contains("browser_native") ||
                                   stream.download_url.contains("/m69/") ||
                                   (!stream.download_url.contains("dash_ln_heaac") &&
                                    !stream.download_url.contains("_audio") &&
                                    stream.download_url.contains(".mp4"));

        // Check EFG metadata for progressive indicators
        let efg_suggests_complete = stream.efg_metadata.contains("xpv_progressive") ||
                                   stream.efg_metadata.contains("progressive") ||
                                   (!stream.efg_metadata.contains("dash_ln_heaac") &&
                                    !stream.efg_metadata.contains("_audio") &&
                                    !stream.efg_metadata.is_empty());

        // Check if quality name suggests it's a complete stream
        let quality_suggests_complete = stream.quality.contains("720p") ||
                                       stream.quality.contains("1080p") ||
                                       stream.quality.contains("480p") ||
                                       stream.quality.contains("540p") ||
                                       stream.quality.contains("360p") ||
                                       stream.quality.contains("HD") ||
                                       stream.quality.contains("Full HD");

        let is_likely_complete = has_video_dimensions && reasonable_size &&
                                (url_suggests_complete || efg_suggests_complete || quality_suggests_complete);

        if is_likely_complete {
            println!("   🔍 Unknown stream heuristics:");
            println!("      📐 Has video dimensions: {} ({}x{})", has_video_dimensions, stream.width, stream.height);
            println!("      📦 Reasonable size: {} ({} MB)", reasonable_size, stream.estimated_size_mb);
            println!("      🌐 URL suggests complete: {}", url_suggests_complete);
            println!("      📄 EFG suggests complete: {}", efg_suggests_complete);
            println!("      🏷️ Quality suggests complete: {}", quality_suggests_complete);
        }

        is_likely_complete
    }

    /// Check if a stream classified as VideoOnly is actually a complete video+audio stream
    /// This handles cases where the classification was too conservative
    fn is_likely_complete_stream_despite_classification(&self, stream: &VideoQuality) -> bool {
        // Check if it has reasonable video dimensions
        let has_video_dimensions = stream.width > 0 && stream.height > 0;

        // Check if it's large enough to potentially contain audio
        // VideoOnly streams are typically much smaller than complete streams
        let size_suggests_complete = stream.estimated_size_mb >= 30;

        // Check if the quality name suggests it's a standard video format
        let quality_suggests_standard_format = stream.quality.contains("720p") ||
                                              stream.quality.contains("1080p") ||
                                              stream.quality.contains("540p") ||
                                              stream.quality.contains("480p") ||
                                              stream.quality.contains("360p") ||
                                              stream.quality.contains("HD") ||
                                              stream.quality.contains("Full HD");

        // Check URL patterns that suggest this is actually a complete stream
        let url_suggests_complete = !stream.download_url.contains("dash_ln_heaac") &&
                                   !stream.download_url.contains("_audio") &&
                                   !stream.download_url.contains("video_only") &&
                                   stream.download_url.contains(".mp4");

        // Check EFG metadata for indicators that this is complete
        let efg_suggests_complete = !stream.efg_metadata.contains("dash_ln_heaac") &&
                                   !stream.efg_metadata.contains("_audio") &&
                                   !stream.efg_metadata.contains("video_only") &&
                                   (stream.efg_metadata.contains("progressive") ||
                                    stream.efg_metadata.contains("xpv_progressive") ||
                                    !stream.efg_metadata.is_empty());

        let is_likely_complete = has_video_dimensions &&
                                size_suggests_complete &&
                                quality_suggests_standard_format &&
                                (url_suggests_complete || efg_suggests_complete);

        if is_likely_complete {
            println!("   🔍 VideoOnly reclassification heuristics:");
            println!("      📐 Has video dimensions: {} ({}x{})", has_video_dimensions, stream.width, stream.height);
            println!("      📦 Size suggests complete: {} ({} MB >= 30)", size_suggests_complete, stream.estimated_size_mb);
            println!("      🏷️ Quality suggests standard: {}", quality_suggests_standard_format);
            println!("      🌐 URL suggests complete: {}", url_suggests_complete);
            println!("      📄 EFG suggests complete: {}", efg_suggests_complete);
        }

        is_likely_complete
    }

    /// Create combined streams from DASH video + audio (for advanced use cases)
    /// This method is kept for potential future use but not used in the simplified downloader
    pub fn create_combined_streams_advanced(
        &self,
        video_streams: &[VideoQuality],
        audio_streams: &[VideoQuality],
    ) -> Vec<VideoQuality> {
        self.create_combined_streams(video_streams, audio_streams)
    }

    /// Create combined video+audio streams from DASH components
    fn create_combined_streams(
        &self,
        video_streams: &[VideoQuality],
        audio_streams: &[VideoQuality],
    ) -> Vec<VideoQuality> {
        if video_streams.is_empty() || audio_streams.is_empty() {
            return Vec::new();
        }

        // Find the best audio stream
        let best_audio = audio_streams
            .iter()
            .max_by_key(|stream| stream.estimated_size_mb)
            .unwrap();

        let mut combined_streams = Vec::new();

        // Create combinations for each video quality
        for video_stream in video_streams {
            if video_stream.width > 0 && video_stream.height > 0 {
                let combined_quality = format!("{} (DASH Combined)", video_stream.quality);
                let combined_size = video_stream.estimated_size_mb + best_audio.estimated_size_mb;

                let combined_stream = VideoQuality {
                    quality: combined_quality,
                    size: format!("~{} MB", combined_size),
                    format: "MP4".to_string(),
                    download_url: format!("COMBINED:{}|{}", video_stream.download_url, best_audio.download_url),
                    width: video_stream.width,
                    height: video_stream.height,
                    stream_type: StreamType::CombinedVideoAudio,
                    efg_metadata: format!("COMBINED:{}+{}", video_stream.efg_metadata, best_audio.efg_metadata),
                    estimated_size_mb: combined_size,
                    bitrate_kbps: None,
                    fps: None,
                    codec: None,
                };

                combined_streams.push(combined_stream);
            }
        }

        combined_streams
    }

    /// Remove duplicates and sort streams by quality
    fn deduplicate_and_sort_streams(&self, streams: Vec<VideoQuality>) -> Vec<VideoQuality> {
        let mut unique_streams = Vec::new();
        let mut seen_qualities = std::collections::HashSet::new();

        for stream in streams {
            let quality_key = format!(
                "{}x{}_{:?}_{}",
                stream.width, stream.height, stream.stream_type, stream.quality
            );

            if !seen_qualities.contains(&quality_key) {
                seen_qualities.insert(quality_key);
                unique_streams.push(stream);
            }
        }

        // Sort by quality (highest resolution first)
        unique_streams.sort_by(|a, b| {
            let a_pixels = a.width * a.height;
            let b_pixels = b.width * b.height;
            b_pixels.cmp(&a_pixels)
        });

        unique_streams
    }
}
