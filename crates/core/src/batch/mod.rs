//! Batch processing functionality
//!
//! This module handles batch operations for processing multiple
//! Facebook videos simultaneously with queue management.

pub mod processor;

pub use processor::{BatchProcessor, BatchJob, BatchProgress, BatchOptions, BatchItem, BatchStatus};
