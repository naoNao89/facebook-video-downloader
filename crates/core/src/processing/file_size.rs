//! Accurate file size detection service for Facebook videos
//!
//! This module provides accurate file size detection by performing actual partial downloads
//! instead of relying on potentially inaccurate HTTP HEAD requests or bitrate estimations.

use crate::common::error::{FacebookExtractorError, Result};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Cache entry for file size information
#[derive(Debug, Clone)]
struct FileSizeCache {
    size_bytes: u64,
    timestamp: std::time::Instant,
    verified: bool, // Whether this was verified through partial download
}

/// Service for accurate file size detection
pub struct AccurateFileSizeService {
    client: Client,
    cache: Arc<Mutex<HashMap<String, FileSizeCache>>>,
    cache_duration: std::time::Duration,
}

impl AccurateFileSizeService {
    /// Create a new accurate file size service
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| FacebookExtractorError::network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_duration: std::time::Duration::from_secs(300), // 5 minutes
        })
    }

    /// Get accurate file size for a video URL
    pub async fn get_accurate_file_size(&self, url: &str) -> Result<u64> {
        // Check cache first
        if let Some(cached_size) = self.get_cached_size(url).await {
            debug!("Using cached file size for URL: {} bytes", cached_size);
            return Ok(cached_size);
        }

        info!("Getting accurate file size for URL: {}", url);

        // Try partial download method first (most accurate)
        match self.get_size_via_partial_download(url).await {
            Ok(size) => {
                self.cache_size(url, size, true).await;
                info!("✅ Accurate file size via partial download: {} bytes ({:.1} MB)", size, size as f64 / 1024.0 / 1024.0);
                return Ok(size);
            }
            Err(e) => {
                warn!("Partial download failed: {}, trying HEAD request", e);
            }
        }

        // Fallback to HEAD request
        match self.get_size_via_head_request(url).await {
            Ok(size) => {
                self.cache_size(url, size, false).await;
                info!("📏 File size via HEAD request: {} bytes ({:.1} MB)", size, size as f64 / 1024.0 / 1024.0);
                Ok(size)
            }
            Err(e) => {
                warn!("HEAD request failed: {}", e);
                Err(FacebookExtractorError::network(format!(
                    "Failed to get file size for URL: {}", e
                )))
            }
        }
    }

    /// Get file size via partial download (most accurate method)
    async fn get_size_via_partial_download(&self, url: &str) -> Result<u64> {
        debug!("Attempting partial download to get accurate file size");

        // Request only the first 1KB to get content-length from actual response
        let response = self.client
            .get(url)
            .header("Range", "bytes=0-1023")
            .send()
            .await
            .map_err(|e| FacebookExtractorError::network(format!("Partial download request failed: {}", e)))?;

        if !response.status().is_success() && response.status().as_u16() != 206 {
            return Err(FacebookExtractorError::network(format!(
                "Partial download failed with status: {}", response.status()
            )));
        }

        // Try to get content-range header first (most reliable for partial content)
        if let Some(content_range) = response.headers().get("content-range") {
            if let Ok(range_str) = content_range.to_str() {
                if let Some(total_size) = self.parse_content_range(range_str) {
                    debug!("Got file size from content-range header: {} bytes", total_size);
                    return Ok(total_size);
                }
            }
        }

        // Fallback to content-length header
        if let Some(content_length) = response.headers().get("content-length") {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(size) = length_str.parse::<u64>() {
                    debug!("Got file size from content-length header: {} bytes", size);
                    return Ok(size);
                }
            }
        }

        Err(FacebookExtractorError::network(
            "No size information available in partial download response".to_string()
        ))
    }

    /// Parse content-range header to extract total file size
    fn parse_content_range(&self, content_range: &str) -> Option<u64> {
        // Content-Range: bytes 0-1023/46726656
        if let Some(slash_pos) = content_range.rfind('/') {
            let total_size_str = &content_range[slash_pos + 1..];
            if total_size_str != "*" {
                return total_size_str.parse::<u64>().ok();
            }
        }
        None
    }

    /// Get file size via HEAD request (fallback method)
    async fn get_size_via_head_request(&self, url: &str) -> Result<u64> {
        debug!("Attempting HEAD request to get file size");

        let response = self.client
            .head(url)
            .send()
            .await
            .map_err(|e| FacebookExtractorError::network(format!("HEAD request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(FacebookExtractorError::network(format!(
                "HEAD request failed with status: {}", response.status()
            )));
        }

        if let Some(content_length) = response.headers().get("content-length") {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(size) = length_str.parse::<u64>() {
                    debug!("Got file size from HEAD request: {} bytes", size);
                    return Ok(size);
                }
            }
        }

        Err(FacebookExtractorError::network(
            "No content-length header in HEAD response".to_string()
        ))
    }

    /// Get cached file size if available and not expired
    async fn get_cached_size(&self, url: &str) -> Option<u64> {
        let cache = self.cache.lock().await;
        if let Some(entry) = cache.get(url) {
            if entry.timestamp.elapsed() < self.cache_duration {
                return Some(entry.size_bytes);
            }
        }
        None
    }

    /// Cache file size information
    async fn cache_size(&self, url: &str, size_bytes: u64, verified: bool) {
        let mut cache = self.cache.lock().await;
        cache.insert(url.to_string(), FileSizeCache {
            size_bytes,
            timestamp: std::time::Instant::now(),
            verified,
        });
    }

    /// Clear expired cache entries
    pub async fn cleanup_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.retain(|_, entry| entry.timestamp.elapsed() < self.cache_duration);
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.lock().await;
        let total = cache.len();
        let verified = cache.values().filter(|entry| entry.verified).count();
        (total, verified)
    }
}

impl Default for AccurateFileSizeService {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback implementation if creation fails
            Self {
                client: Client::new(),
                cache: Arc::new(Mutex::new(HashMap::new())),
                cache_duration: std::time::Duration::from_secs(300),
            }
        })
    }
}

/// Convert bytes to megabytes
pub fn bytes_to_mb(bytes: u64) -> u32 {
    (bytes / 1024 / 1024) as u32
}

/// Convert megabytes to bytes
pub fn mb_to_bytes(mb: u32) -> u64 {
    (mb as u64) * 1024 * 1024
}
