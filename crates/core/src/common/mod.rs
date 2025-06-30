//! Common utilities and shared functionality
//!
//! This module contains shared types, error handling, configuration,
//! and utility functions used across the entire core library.

pub mod config;
pub mod error;
pub mod types;
pub mod utils;

pub use config::ExtractorConfig;
pub use error::{FacebookExtractorError, Result};
pub use types::*;
pub use utils::*;
