//! Video extraction functionality
//!
//! This module contains all the core logic for extracting video information
//! from Facebook URLs, including metadata parsing and stream detection.

pub mod extractor;
pub mod metadata;
pub mod streams;

pub use extractor::FacebookExtractor;
pub use metadata::*;
pub use streams::*;
