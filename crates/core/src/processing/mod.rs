//! Post-extraction processing functionality
//!
//! This module handles video processing tasks such as compression,
//! thumbnail generation, and file size calculations.

pub mod compression;
pub mod thumbnail;
pub mod file_size;

pub use compression::{CompressionService, CompressionOptions, CompressionQuality, CompressionResult, CompressionProgress};
pub use thumbnail::ThumbnailExtractor;
pub use crate::common::types::{ThumbnailVariant, ThumbnailCollection, AspectRatio, DisplayContext};
pub use file_size::*;
