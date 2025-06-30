//! Utility functions for the Facebook extractor

/// URL cleaning utilities
pub struct UrlCleaner;

impl UrlCleaner {
    /// Create a new URL cleaner
    pub fn new() -> Self {
        Self
    }

    /// Decode HTML entities
    pub fn decode_html_entities(&self, input: &str) -> String {
        let mut result = input.to_string();
        
        // Replace common HTML entities
        result = result.replace("&amp;", "&");
        result = result.replace("&quot;", "\"");
        result = result.replace("&#039;", "'");
        result = result.replace("&lt;", "<");
        result = result.replace("&gt;", ">");
        
        // Replace Unicode escape sequences
        result = result.replace("\\u0026", "&");
        result = result.replace("\\u003C", "<");
        result = result.replace("\\u003E", ">");
        result = result.replace("\\u0022", "\"");
        result = result.replace("\\u0027", "'");
        
        result
    }

    /// Fix escaped JSON
    pub fn fix_escaped_json(&self, input: &str) -> String {
        let mut result = input.to_string();
        
        // Remove escaped slashes
        result = result.replace("\\/", "/");
        result = result.replace("\\\\", "\\");
        
        result
    }

    /// Clean Facebook URL by removing HTML entities and malformed parts
    pub fn clean_facebook_url(&self, url: &str) -> String {
        let mut clean_url = url.to_string();

        // First, remove any DASH manifest XML contamination
        if let Some(xml_start) = clean_url.find("u003C") {
            clean_url = clean_url[..xml_start].to_string();
        }

        // Remove any pipe-separated URLs (combined stream artifacts)
        if let Some(pipe_pos) = clean_url.find("|https://") {
            clean_url = clean_url[..pipe_pos].to_string();
        }

        // Replace HTML entities
        clean_url = self.decode_html_entities(&clean_url);

        // Remove escaped slashes
        clean_url = self.fix_escaped_json(&clean_url);

        // Handle query parameters after .mp4
        if let Some(mp4_pos) = clean_url.find(".mp4") {
            let end_pos = mp4_pos + 4;
            // Look for query parameters that are valid
            if let Some(query_start) = clean_url[end_pos..].find('?') {
                let query_part = &clean_url[end_pos + query_start..];
                // Keep essential Facebook parameters
                if query_part.contains("efg=")
                    || query_part.contains("_nc_")
                    || query_part.contains("oh=")
                    || query_part.contains("oe=")
                    || query_part.contains("strext=")
                    || query_part.contains("ccb=")
                    || query_part.contains("vs=")
                {
                    // Keep the URL as is - these are essential parameters
                } else {
                    // Only truncate if no essential parameters found
                    clean_url = clean_url[..end_pos].to_string();
                }
            } else {
                // Look for any non-URL characters after .mp4, but be more permissive
                let after_mp4 = &clean_url[end_pos..];
                if let Some(bad_char_pos) = after_mp4.find(|c: char| {
                    !c.is_ascii_alphanumeric() && !"?&=_-.%".contains(c)
                }) {
                    clean_url = clean_url[..end_pos + bad_char_pos].to_string();
                }
            }
        }

        // Remove any leading/trailing quotes or whitespace
        clean_url = clean_url.trim_matches('"').trim().to_string();

        // If URL doesn't start with https://, try to fix it
        if !clean_url.starts_with("https://") && clean_url.contains("https://") {
            if let Some(https_pos) = clean_url.find("https://") {
                clean_url = clean_url[https_pos..].to_string();
            }
        }

        clean_url
    }
}

/// Filename sanitization utilities
pub struct FilenameSanitizer;

impl FilenameSanitizer {
    /// Create a new filename sanitizer
    pub fn new() -> Self {
        Self
    }

    /// Sanitize filename for cross-platform compatibility
    pub fn sanitize_filename(&self, filename: &str) -> String {
        let mut sanitized = filename.to_string();

        // Remove or replace invalid characters
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\\', '/'];
        for &ch in &invalid_chars {
            sanitized = sanitized.replace(ch, "_");
        }

        // Remove control characters
        sanitized = sanitized.chars().filter(|c| !c.is_control()).collect();

        // Handle reserved Windows names
        let reserved_names = [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5",
            "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4",
            "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];

        for &reserved in &reserved_names {
            if sanitized.to_uppercase() == reserved {
                sanitized = format!("{}_", sanitized);
                break;
            }
        }

        // Remove leading/trailing dots and spaces
        sanitized = sanitized.trim_matches('.').trim().to_string();

        // Ensure filename is not empty
        if sanitized.is_empty() {
            sanitized = "untitled".to_string();
        }

        // Limit length
        if sanitized.len() > 200 {
            sanitized = sanitized[..200].to_string();
        }

        sanitized
    }

    /// Create a descriptive filename from video metadata
    pub fn create_descriptive_filename(
        &self,
        title: &str,
        author: &str,
        quality: &str,
        video_id: &str,
        extension: &str,
    ) -> String {
        // Clean individual components
        let clean_title = self.sanitize_filename(title);
        let clean_author = self.sanitize_filename(author);
        let clean_quality = self.sanitize_filename(quality);

        // Build filename
        let filename = if !clean_title.is_empty() && clean_title != "untitled" {
            if !clean_author.is_empty() && clean_author != "Unknown Author" {
                format!("{} - {} - {}", clean_title, clean_author, clean_quality)
            } else {
                format!("{} - {}", clean_title, clean_quality)
            }
        } else {
            format!("Facebook_Video_{} - {}", video_id, clean_quality)
        };

        // Add extension
        format!("{}.{}", self.sanitize_filename(&filename), extension)
    }
}

/// Progress tracking utilities
pub struct ProgressTracker {
    start_time: std::time::Instant,
    last_update: std::time::Instant,
    last_bytes: u64,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            start_time: now,
            last_update: now,
            last_bytes: 0,
        }
    }

    /// Update progress and calculate speed/ETA
    pub fn update(&mut self, downloaded_bytes: u64, total_bytes: Option<u64>) -> ProgressInfo {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.start_time);
        let since_last_update = now.duration_since(self.last_update);

        // Calculate speed
        let speed_bytes_per_sec = if since_last_update.as_secs_f64() > 0.0 {
            let bytes_diff = downloaded_bytes.saturating_sub(self.last_bytes);
            (bytes_diff as f64 / since_last_update.as_secs_f64()) as u64
        } else {
            0
        };

        // Calculate ETA
        let eta_seconds = if let Some(total) = total_bytes {
            if speed_bytes_per_sec > 0 && downloaded_bytes < total {
                let remaining_bytes = total - downloaded_bytes;
                Some(remaining_bytes / speed_bytes_per_sec)
            } else {
                None
            }
        } else {
            None
        };

        // Calculate progress percentage
        let progress = if let Some(total) = total_bytes {
            if total > 0 {
                (downloaded_bytes as f64 / total as f64 * 100.0).min(100.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Update tracking state
        self.last_update = now;
        self.last_bytes = downloaded_bytes;

        ProgressInfo {
            progress,
            downloaded_bytes,
            total_bytes,
            speed_bytes_per_sec: if speed_bytes_per_sec > 0 { Some(speed_bytes_per_sec) } else { None },
            eta_seconds,
            elapsed_seconds: elapsed.as_secs(),
        }
    }
}

/// Progress information
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub progress: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed_bytes_per_sec: Option<u64>,
    pub eta_seconds: Option<u64>,
    pub elapsed_seconds: u64,
}

/// Format bytes into human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format duration into human-readable string
pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
