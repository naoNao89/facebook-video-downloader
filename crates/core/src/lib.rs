//! # Facebook Video Extractor Core Library
//!
//! A comprehensive library for extracting video information and downloading content from Facebook.
//! This library provides the core functionality for the Facebook Video Downloader application.
//!
//! ## Features
//! - Extract video information from Facebook URLs
//! - Support for regular videos, Reels, and private content
//! - Download videos with progress tracking
//! - Metadata extraction and parsing
//! - Stream quality analysis and selection
//! - FFmpeg integration for DASH stream combination
//!
//! ## Usage
//! ```rust
//! use facebook_extractor_core::{FacebookExtractor, ExtractorConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let extractor = FacebookExtractor::new()?;
//!     let video_info = extractor.extract_video_info("https://facebook.com/watch?v=123").await?;
//!     println!("Video title: {}", video_info.title);
//!     Ok(())
//! }
//! ```

// Core modules organized by functionality
pub mod common;
pub mod extraction;
pub mod network;
pub mod processing;
pub mod batch;

#[cfg(test)]
pub mod tests;

// Re-export main types and functions for convenience
pub use common::{FacebookExtractorError, Result, ExtractorConfig};
pub use extraction::FacebookExtractor;
pub use common::{VideoInfo, VideoQuality, VideoMetadata};
pub use network::{AntiBlockingManager, AntiBlockingConfig, DownloadManager};
pub use processing::{CompressionService, ThumbnailExtractor, CompressionQuality};
pub use batch::{BatchProcessor, BatchJob, BatchProgress};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// User agent strings for browser simulation
pub const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0",
];

/// Default configuration constants
pub mod constants {
    use std::time::Duration;
    
    pub const DEFAULT_TIMEOUT_SECS: u64 = 600;
    pub const CONNECTION_TIMEOUT_SECS: u64 = 30;
    pub const MAX_RETRY_ATTEMPTS: usize = 5;
    pub const MAX_CONCURRENT_DOWNLOADS: usize = 6;
    pub const MIN_FILE_SIZE_MB: u64 = 5;
    pub const MAX_FILENAME_LENGTH: usize = 95;
    
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(DEFAULT_TIMEOUT_SECS);
    pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(CONNECTION_TIMEOUT_SECS);
}
