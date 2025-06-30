//! Network-related functionality
//!
//! This module handles all network operations including downloads,
//! anti-blocking strategies, and HTTP client management.

pub mod anti_blocking;
pub mod download;
pub mod real_ipv6;

pub use anti_blocking::{AntiBlockingManager, AntiBlockingConfig, AntiBlockingStats};
pub use download::{DownloadManager, DownloadProgress, DownloadTask, DownloadStatus};
