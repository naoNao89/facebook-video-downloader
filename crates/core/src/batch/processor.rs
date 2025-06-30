//! Batch processing module for handling multiple video downloads simultaneously
//!
//! This module provides functionality to process multiple Facebook video URLs
//! in batches with queue management, progress tracking, and error handling.

use crate::{
    common::{error::{FacebookExtractorError, Result}, types::VideoInfo},
    extraction::extractor::FacebookExtractor,
    network::{
        download::{DownloadManager, DownloadTask, DownloadProgress, DownloadStatus},
        anti_blocking::{AntiBlockingConfig, AntiBlockingManager}
    },
    processing::compression::{CompressionService, CompressionOptions, CompressionQuality, CompressionResult},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Status of a batch processing job
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchStatus {
    /// Batch is queued and waiting to start
    Queued,
    /// Batch is currently being processed
    Processing,
    /// Batch has completed successfully
    Completed,
    /// Batch was paused by user
    Paused,
    /// Batch was cancelled by user
    Cancelled,
    /// Batch failed due to errors
    Failed,
}

/// Status of an individual item within a batch
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchItemStatus {
    /// Item is queued and waiting to be extracted
    Queued,
    /// Item is being validated
    Validating,
    /// Item is being extracted
    Extracting,
    /// Item extraction completed and is ready for download
    ReadyForDownload,
    /// Item is being downloaded
    Downloading,
    /// Item is being compressed (if compression enabled)
    Compressing,
    /// Item completed successfully
    Completed,
    /// Item failed with error
    Failed,
    /// Item was skipped due to validation failure
    Skipped,
    /// Item was cancelled
    Cancelled,
}

/// Individual item in a batch processing job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    /// Unique identifier for this item
    pub id: String,
    /// Original URL provided by user
    pub url: String,
    /// Current status of this item
    pub status: BatchItemStatus,
    /// Extracted video information (if extraction succeeded)
    pub video_info: Option<VideoInfo>,
    /// Selected quality index for download
    pub quality_index: Option<usize>,
    /// Download progress (if downloading)
    pub download_progress: Option<DownloadProgress>,
    /// Compression result (if compression was applied)
    pub compression_result: Option<CompressionResult>,
    /// Error message if item failed
    pub error_message: Option<String>,
    /// When this item was created
    pub created_at: DateTime<Utc>,
    /// When this item started processing
    pub started_at: Option<DateTime<Utc>>,
    /// When this item completed (success or failure)
    pub completed_at: Option<DateTime<Utc>>,
    /// Output file path (if download succeeded)
    pub output_path: Option<String>,
}

impl BatchItem {
    /// Create a new batch item from a URL
    pub fn new(url: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            url,
            status: BatchItemStatus::Queued,
            video_info: None,
            quality_index: None,
            download_progress: None,
            compression_result: None,
            error_message: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            output_path: None,
        }
    }

    /// Check if this item is in a finished state
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            BatchItemStatus::Completed
                | BatchItemStatus::Failed
                | BatchItemStatus::Skipped
                | BatchItemStatus::Cancelled
        )
    }

    /// Check if this item is currently being processed
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            BatchItemStatus::Validating
                | BatchItemStatus::Extracting
                | BatchItemStatus::Downloading
                | BatchItemStatus::Compressing
        )
    }

    /// Mark item as failed with error message
    pub fn mark_failed(&mut self, error: String) {
        self.status = BatchItemStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
    }

    /// Mark item as completed successfully
    pub fn mark_completed(&mut self, output_path: Option<String>) {
        self.status = BatchItemStatus::Completed;
        self.output_path = output_path;
        self.completed_at = Some(Utc::now());
    }
}

/// Options for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOptions {
    /// Maximum number of concurrent downloads
    pub max_concurrent_downloads: usize,
    /// Maximum number of concurrent extractions (recommended: 1 to avoid Facebook rate limiting)
    pub max_concurrent_extractions: usize,
    /// Default quality preference (index into quality array)
    pub default_quality_index: Option<usize>,
    /// Whether to enable compression for all downloads
    pub enable_compression: bool,
    /// Compression quality to use if compression is enabled
    pub compression_quality: CompressionQuality,
    /// Whether to preserve original files when compressing
    pub preserve_original_on_compression: bool,
    /// Output directory for downloads
    pub output_directory: Option<String>,
    /// Whether to retry failed items
    pub retry_failed_items: bool,
    /// Maximum number of retries per item
    pub max_retries_per_item: usize,
    /// Whether to continue processing if some items fail
    pub continue_on_errors: bool,
    /// Anti-blocking configuration
    pub anti_blocking_config: Option<AntiBlockingConfig>,
}

impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: 3,
            max_concurrent_extractions: 1, // Reduced to 1 to avoid Facebook rate limiting
            default_quality_index: None, // Use best available quality
            enable_compression: false,
            compression_quality: CompressionQuality::Medium,
            preserve_original_on_compression: false,
            output_directory: None,
            retry_failed_items: true,
            max_retries_per_item: 2,
            continue_on_errors: true,
            anti_blocking_config: None,
        }
    }
}

/// Progress information for a batch processing job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    /// Unique identifier for the batch
    pub id: String,
    /// Current status of the batch
    pub status: BatchStatus,
    /// Total number of items in the batch
    pub total_items: usize,
    /// Number of items completed successfully
    pub completed_items: usize,
    /// Number of items that failed
    pub failed_items: usize,
    /// Number of items currently being processed
    pub active_items: usize,
    /// Number of items still queued
    pub queued_items: usize,
    /// Overall progress percentage (0.0 to 100.0)
    pub progress_percentage: f64,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,
    /// Current processing speed (items per minute)
    pub items_per_minute: Option<f64>,
    /// When the batch started processing
    pub started_at: Option<DateTime<Utc>>,
    /// When the batch completed (if finished)
    pub completed_at: Option<DateTime<Utc>>,
    /// Total bytes downloaded across all items
    pub total_downloaded_bytes: u64,
    /// Current download speed across all active downloads
    pub current_download_speed_bps: Option<u64>,
}

/// Statistics for a completed or ongoing batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatistics {
    /// Total processing time in seconds
    pub total_processing_time_seconds: u64,
    /// Average time per item in seconds
    pub average_time_per_item_seconds: f64,
    /// Success rate as percentage
    pub success_rate_percentage: f64,
    /// Total data downloaded in MB
    pub total_downloaded_mb: u64,
    /// Total data saved through compression in MB
    pub total_compression_savings_mb: u64,
    /// Most common failure reason
    pub most_common_failure_reason: Option<String>,
}

/// A batch processing job containing multiple video URLs to process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJob {
    /// Unique identifier for this batch
    pub id: String,
    /// Human-readable name for this batch
    pub name: String,
    /// Current status of the batch
    pub status: BatchStatus,
    /// List of items to process
    pub items: Vec<BatchItem>,
    /// Processing options for this batch
    pub options: BatchOptions,
    /// Current progress information
    pub progress: BatchProgress,
    /// When this batch was created
    pub created_at: DateTime<Utc>,
    /// When this batch started processing
    pub started_at: Option<DateTime<Utc>>,
    /// When this batch completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Retry counts for each item (item_id -> retry_count)
    pub retry_counts: HashMap<String, usize>,
}

impl BatchJob {
    /// Create a new batch job from a list of URLs
    pub fn new(name: String, urls: Vec<String>, options: BatchOptions) -> Self {
        let id = Uuid::new_v4().to_string();
        let items: Vec<BatchItem> = urls.into_iter().map(BatchItem::new).collect();
        let total_items = items.len();

        let progress = BatchProgress {
            id: id.clone(),
            status: BatchStatus::Queued,
            total_items,
            completed_items: 0,
            failed_items: 0,
            active_items: 0,
            queued_items: total_items,
            progress_percentage: 0.0,
            eta_seconds: None,
            items_per_minute: None,
            started_at: None,
            completed_at: None,
            total_downloaded_bytes: 0,
            current_download_speed_bps: None,
        };

        Self {
            id,
            name,
            status: BatchStatus::Queued,
            items,
            options,
            progress,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            retry_counts: HashMap::new(),
        }
    }

    /// Update progress based on current item states
    pub fn update_progress(&mut self) {
        let total = self.items.len();
        let completed = self.items.iter().filter(|item| item.status == BatchItemStatus::Completed).count();
        let failed = self.items.iter().filter(|item| item.status == BatchItemStatus::Failed).count();
        let active = self.items.iter().filter(|item| item.is_active()).count();
        let queued = self.items.iter().filter(|item|
            matches!(item.status, BatchItemStatus::Queued | BatchItemStatus::ReadyForDownload)
        ).count();

        let old_progress = self.progress.progress_percentage;

        self.progress.total_items = total;
        self.progress.completed_items = completed;
        self.progress.failed_items = failed;
        self.progress.active_items = active;
        self.progress.queued_items = queued;

        // Calculate overall progress percentage
        let finished_items = completed + failed;
        self.progress.progress_percentage = if total > 0 {
            (finished_items as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // Log progress updates for debugging
        if (self.progress.progress_percentage - old_progress).abs() > 0.1 {
            tracing::info!("📊 Batch '{}' progress: {:.1}% ({}/{} completed, {} active, {} failed)",
                self.name, self.progress.progress_percentage, completed, total, active, failed);
        }

        // Update batch status based on item states
        if finished_items == total {
            self.status = if failed == 0 {
                BatchStatus::Completed
            } else if completed == 0 {
                BatchStatus::Failed
            } else {
                BatchStatus::Completed // Partial success still counts as completed
            };
            self.progress.status = self.status.clone();
            if self.completed_at.is_none() {
                self.completed_at = Some(Utc::now());
                self.progress.completed_at = self.completed_at;
            }
        } else if active > 0 {
            self.status = BatchStatus::Processing;
            self.progress.status = self.status.clone();
        }

        // Calculate total downloaded bytes
        self.progress.total_downloaded_bytes = self.items
            .iter()
            .filter_map(|item| item.download_progress.as_ref())
            .map(|progress| progress.downloaded_bytes)
            .sum();

        // Calculate current download speed
        let active_speeds: Vec<u64> = self.items
            .iter()
            .filter_map(|item| item.download_progress.as_ref())
            .filter_map(|progress| progress.speed_bytes_per_sec)
            .collect();

        self.progress.current_download_speed_bps = if !active_speeds.is_empty() {
            Some(active_speeds.iter().sum())
        } else {
            None
        };
    }

    /// Get statistics for this batch
    pub fn get_statistics(&self) -> BatchStatistics {
        let total_time = if let (Some(started), Some(completed)) = (self.started_at, self.completed_at) {
            (completed - started).num_seconds() as u64
        } else if let Some(started) = self.started_at {
            (Utc::now() - started).num_seconds() as u64
        } else {
            0
        };

        let completed_count = self.progress.completed_items;
        let total_count = self.progress.total_items;

        let average_time = if completed_count > 0 {
            total_time as f64 / completed_count as f64
        } else {
            0.0
        };

        let success_rate = if total_count > 0 {
            (completed_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };

        let total_downloaded_mb = self.progress.total_downloaded_bytes / 1024 / 1024;

        let total_compression_savings_mb = self.items
            .iter()
            .filter_map(|item| item.compression_result.as_ref())
            .map(|result| result.original_size_mb.saturating_sub(result.compressed_size_mb))
            .sum();

        // Find most common failure reason
        let mut failure_counts: HashMap<String, usize> = HashMap::new();
        for item in &self.items {
            if let Some(error) = &item.error_message {
                *failure_counts.entry(error.clone()).or_insert(0) += 1;
            }
        }

        let most_common_failure_reason = failure_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(reason, _)| reason);

        BatchStatistics {
            total_processing_time_seconds: total_time,
            average_time_per_item_seconds: average_time,
            success_rate_percentage: success_rate,
            total_downloaded_mb,
            total_compression_savings_mb,
            most_common_failure_reason,
        }
    }

    /// Check if this batch is in a finished state
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            BatchStatus::Completed | BatchStatus::Failed | BatchStatus::Cancelled
        )
    }

    /// Check if this batch can be resumed
    pub fn can_resume(&self) -> bool {
        matches!(self.status, BatchStatus::Paused) ||
        (matches!(self.status, BatchStatus::Failed) && self.options.retry_failed_items)
    }
}

/// Main batch processor for handling multiple video downloads
pub struct BatchProcessor {
    /// Facebook extractor for video information extraction
    extractor: Arc<FacebookExtractor>,
    /// Download manager for handling downloads
    download_manager: Arc<DownloadManager>,
    /// Compression service for video compression
    compression_service: Arc<CompressionService>,
    /// Currently active batch jobs
    active_batches: Arc<RwLock<HashMap<String, BatchJob>>>,
    /// Global processing options
    global_options: Arc<RwLock<BatchOptions>>,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub async fn new() -> Result<Self> {
        let extractor = Arc::new(FacebookExtractor::new()?);
        let download_manager = Arc::new(DownloadManager::new(3)); // Default max concurrent

        // Try to create compression service, but don't fail if FFmpeg is not available
        let compression_service = Arc::new(
            CompressionService::new().unwrap_or_else(|e| {
                tracing::warn!("Compression service unavailable: {}. Batch processing will continue without compression support.", e);
                CompressionService::default()
            })
        );

        Ok(Self {
            extractor,
            download_manager,
            compression_service,
            active_batches: Arc::new(RwLock::new(HashMap::new())),
            global_options: Arc::new(RwLock::new(BatchOptions::default())),
        })
    }

    /// Create a new batch processor with custom options
    pub async fn with_options(options: BatchOptions) -> Result<Self> {
        let processor = Self::new().await?;
        *processor.global_options.write().await = options;
        Ok(processor)
    }

    /// Start processing a new batch
    pub async fn start_batch(&self, name: String, urls: Vec<String>, options: Option<BatchOptions>) -> Result<String> {
        // Clean up any stuck downloads from previous sessions
        self.download_manager.cleanup_stuck_downloads().await;
        self.download_manager.cleanup_finished_tasks().await;

        let batch_options = options.unwrap_or_else(|| {
            // Use global options as default
            futures::executor::block_on(async {
                self.global_options.read().await.clone()
            })
        });

        let mut batch = BatchJob::new(name, urls, batch_options);
        batch.status = BatchStatus::Processing;
        batch.started_at = Some(Utc::now());
        batch.progress.started_at = batch.started_at;

        let batch_id = batch.id.clone();

        // Add to active batches
        {
            let mut active_batches = self.active_batches.write().await;
            active_batches.insert(batch_id.clone(), batch);
        }

        // Start processing in background
        let processor = self.clone();
        let batch_id_clone = batch_id.clone();
        tokio::spawn(async move {
            if let Err(e) = processor.process_batch_internal(batch_id_clone).await {
                tracing::error!("Batch processing failed: {}", e);
            }
        });

        Ok(batch_id)
    }

    /// Get progress for a specific batch
    pub async fn get_batch_progress(&self, batch_id: &str) -> Option<BatchProgress> {
        let active_batches = self.active_batches.read().await;
        active_batches.get(batch_id).map(|batch| batch.progress.clone())
    }

    /// Get full batch information
    pub async fn get_batch(&self, batch_id: &str) -> Option<BatchJob> {
        let active_batches = self.active_batches.read().await;
        active_batches.get(batch_id).cloned()
    }

    /// Get all active batches
    pub async fn get_all_batches(&self) -> Vec<BatchJob> {
        let active_batches = self.active_batches.read().await;
        active_batches.values().cloned().collect()
    }

    /// Pause a batch
    pub async fn pause_batch(&self, batch_id: &str) -> Result<()> {
        let mut active_batches = self.active_batches.write().await;
        if let Some(batch) = active_batches.get_mut(batch_id) {
            if batch.status == BatchStatus::Processing {
                batch.status = BatchStatus::Paused;
                batch.progress.status = BatchStatus::Paused;
                Ok(())
            } else {
                Err(FacebookExtractorError::batch(format!(
                    "Cannot pause batch in status: {:?}",
                    batch.status
                )))
            }
        } else {
            Err(FacebookExtractorError::batch(format!(
                "Batch not found: {}",
                batch_id
            )))
        }
    }

    /// Resume a paused batch
    pub async fn resume_batch(&self, batch_id: &str) -> Result<()> {
        {
            let mut active_batches = self.active_batches.write().await;
            if let Some(batch) = active_batches.get_mut(batch_id) {
                if !batch.can_resume() {
                    return Err(FacebookExtractorError::batch(format!(
                        "Cannot resume batch in status: {:?}",
                        batch.status
                    )));
                }
                batch.status = BatchStatus::Processing;
                batch.progress.status = BatchStatus::Processing;
            } else {
                return Err(FacebookExtractorError::batch(format!(
                    "Batch not found: {}",
                    batch_id
                )));
            }
        }

        // Restart processing
        let processor = self.clone();
        let batch_id_clone = batch_id.to_string();
        tokio::spawn(async move {
            if let Err(e) = processor.process_batch_internal(batch_id_clone).await {
                tracing::error!("Batch processing failed on resume: {}", e);
            }
        });

        Ok(())
    }

    /// Cancel a batch
    pub async fn cancel_batch(&self, batch_id: &str) -> Result<()> {
        let mut active_batches = self.active_batches.write().await;
        if let Some(batch) = active_batches.get_mut(batch_id) {
            batch.status = BatchStatus::Cancelled;
            batch.progress.status = BatchStatus::Cancelled;
            batch.completed_at = Some(Utc::now());
            batch.progress.completed_at = batch.completed_at;

            // Cancel all active items
            for item in &mut batch.items {
                if item.is_active() {
                    item.status = BatchItemStatus::Cancelled;
                    item.completed_at = Some(Utc::now());
                }
            }

            batch.update_progress();
            Ok(())
        } else {
            Err(FacebookExtractorError::batch(format!(
                "Batch not found: {}",
                batch_id
            )))
        }
    }

    /// Remove a completed batch from active batches
    pub async fn remove_batch(&self, batch_id: &str) -> Result<BatchJob> {
        let mut active_batches = self.active_batches.write().await;
        active_batches.remove(batch_id).ok_or_else(|| {
            FacebookExtractorError::batch(format!("Batch not found: {}", batch_id))
        })
    }

    /// Clean up finished batches
    pub async fn cleanup_finished_batches(&self) {
        let mut active_batches = self.active_batches.write().await;
        active_batches.retain(|_, batch| !batch.is_finished());
    }
}

// Clone implementation for BatchProcessor to enable spawning async tasks
impl Clone for BatchProcessor {
    fn clone(&self) -> Self {
        Self {
            extractor: Arc::clone(&self.extractor),
            download_manager: Arc::clone(&self.download_manager),
            compression_service: Arc::clone(&self.compression_service),
            active_batches: Arc::clone(&self.active_batches),
            global_options: Arc::clone(&self.global_options),
        }
    }
}

impl BatchProcessor {
    /// Internal method to process a batch
    async fn process_batch_internal(&self, batch_id: String) -> Result<()> {
        loop {
            // Check if batch is still active and not paused/cancelled
            let should_continue = {
                let active_batches = self.active_batches.read().await;
                if let Some(batch) = active_batches.get(&batch_id) {
                    matches!(batch.status, BatchStatus::Processing)
                } else {
                    false
                }
            };

            if !should_continue {
                break;
            }

            // Clean up stuck downloads periodically
            self.download_manager.cleanup_stuck_downloads().await;

            // Process next available items
            let processed_any = self.process_next_items(&batch_id).await?;

            // Update batch progress
            self.update_batch_progress(&batch_id).await?;

            // Check if batch is complete
            let is_complete = {
                let active_batches = self.active_batches.read().await;
                if let Some(batch) = active_batches.get(&batch_id) {
                    batch.items.iter().all(|item| item.is_finished())
                } else {
                    true
                }
            };

            if is_complete {
                self.finalize_batch(&batch_id).await?;
                break;
            }

            // If no items were processed, wait a bit before checking again
            if !processed_any {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }

        Ok(())
    }

    /// Process the next available items in the batch
    async fn process_next_items(&self, batch_id: &str) -> Result<bool> {
        let mut processed_any = false;

        // Get batch options and current state
        let (max_concurrent_extractions, max_concurrent_downloads) = {
            let active_batches = self.active_batches.read().await;
            if let Some(batch) = active_batches.get(batch_id) {
                (
                    batch.options.max_concurrent_extractions,
                    batch.options.max_concurrent_downloads,
                )
            } else {
                return Ok(false);
            }
        };

        // Count current active operations
        let (current_extractions, current_downloads) = {
            let active_batches = self.active_batches.read().await;
            if let Some(batch) = active_batches.get(batch_id) {
                let extractions = batch.items.iter()
                    .filter(|item| matches!(item.status, BatchItemStatus::Validating | BatchItemStatus::Extracting))
                    .count();
                let downloads = batch.items.iter()
                    .filter(|item| matches!(item.status, BatchItemStatus::Downloading | BatchItemStatus::Compressing))
                    .count();
                (extractions, downloads)
            } else {
                return Ok(false);
            }
        };

        // Start new extractions if we have capacity
        if current_extractions < max_concurrent_extractions {
            let items_to_extract = self.get_items_for_extraction(batch_id, max_concurrent_extractions - current_extractions).await;
            if !items_to_extract.is_empty() {
                tracing::info!("🔍 Starting extraction for {} items in batch {} (current extractions: {}/{})",
                    items_to_extract.len(), batch_id, current_extractions, max_concurrent_extractions);
            }
            for item_id in items_to_extract {
                self.start_item_extraction(batch_id, &item_id).await?;
                processed_any = true;
            }
        }

        // Start new downloads if we have capacity
        if current_downloads < max_concurrent_downloads {
            let items_to_download = self.get_items_for_download(batch_id, max_concurrent_downloads - current_downloads).await;
            if !items_to_download.is_empty() {
                tracing::debug!("Found {} items ready for download in batch {}", items_to_download.len(), batch_id);
            }
            for item_id in items_to_download {
                match self.start_item_download(batch_id, &item_id).await {
                    Ok(()) => {
                        processed_any = true;
                    }
                    Err(e) => {
                        tracing::error!("Failed to start download for item {}: {}", item_id, e);
                        // Continue with other items
                    }
                }
            }
        }

        Ok(processed_any)
    }

    /// Get items that are ready for extraction
    async fn get_items_for_extraction(&self, batch_id: &str, max_count: usize) -> Vec<String> {
        let active_batches = self.active_batches.read().await;
        if let Some(batch) = active_batches.get(batch_id) {
            batch.items
                .iter()
                .filter(|item| item.status == BatchItemStatus::Queued)
                .take(max_count)
                .map(|item| item.id.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get items that are ready for download
    async fn get_items_for_download(&self, batch_id: &str, max_count: usize) -> Vec<String> {
        let active_batches = self.active_batches.read().await;
        if let Some(batch) = active_batches.get(batch_id) {
            batch.items
                .iter()
                .filter(|item| {
                    // Item must be in ReadyForDownload status with complete extraction data
                    item.status == BatchItemStatus::ReadyForDownload &&
                    item.video_info.is_some() &&
                    item.quality_index.is_some()
                })
                .take(max_count)
                .map(|item| item.id.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Start extraction for a specific item
    async fn start_item_extraction(&self, batch_id: &str, item_id: &str) -> Result<()> {
        // Update item status to validating
        {
            let mut active_batches = self.active_batches.write().await;
            if let Some(batch) = active_batches.get_mut(batch_id) {
                if let Some(item) = batch.items.iter_mut().find(|i| i.id == item_id) {
                    item.status = BatchItemStatus::Validating;
                    item.started_at = Some(Utc::now());
                }
            }
        }

        // Get URL for extraction
        let url = {
            let active_batches = self.active_batches.read().await;
            if let Some(batch) = active_batches.get(batch_id) {
                if let Some(item) = batch.items.iter().find(|i| i.id == item_id) {
                    item.url.clone()
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        };

        // Spawn extraction task
        let extractor = Arc::clone(&self.extractor);
        let processor = self.clone();
        let batch_id = batch_id.to_string();
        let item_id = item_id.to_string();

        tokio::spawn(async move {
            processor.extract_item_async(batch_id, item_id, url, extractor).await;
        });

        Ok(())
    }

    /// Async extraction for an item
    async fn extract_item_async(
        &self,
        batch_id: String,
        item_id: String,
        url: String,
        extractor: Arc<FacebookExtractor>,
    ) {
        // Update status to extracting
        {
            let mut active_batches = self.active_batches.write().await;
            if let Some(batch) = active_batches.get_mut(&batch_id) {
                if let Some(item) = batch.items.iter_mut().find(|i| i.id == item_id) {
                    item.status = BatchItemStatus::Extracting;
                }
            }
        }

        // Add a small delay to avoid overwhelming Facebook servers with concurrent requests
        // This helps prevent rate limiting and anti-bot measures
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // Perform extraction
        match extractor.extract_video_info(&url).await {
            Ok(video_info) => {
                tracing::info!("Successfully extracted video info for item {} ({})", item_id, video_info.title);
                tracing::debug!("Video info details: title='{}', author='{}', qualities={}, content_type={:?}, privacy_level={:?}",
                    video_info.title,
                    video_info.metadata.author,
                    video_info.qualities.len(),
                    video_info.content_type,
                    video_info.privacy_level
                );

                // Determine quality index - validate against available qualities
                let quality_index = {
                    let active_batches = self.active_batches.read().await;
                    let default_index = if let Some(batch) = active_batches.get(&batch_id) {
                        batch.options.default_quality_index.unwrap_or(0)
                    } else {
                        0
                    };

                    // Validate that the quality index is valid for this video
                    if default_index < video_info.qualities.len() {
                        Some(default_index)
                    } else if !video_info.qualities.is_empty() {
                        // Use the best available quality if default is invalid
                        Some(0)
                    } else {
                        // No qualities available - this item cannot be downloaded
                        None
                    }
                };

                // Update item with extracted info
                {
                    let mut active_batches = self.active_batches.write().await;
                    if let Some(batch) = active_batches.get_mut(&batch_id) {
                        if let Some(item) = batch.items.iter_mut().find(|i| i.id == item_id) {
                            item.video_info = Some(video_info.clone());

                            if let Some(qi) = quality_index {
                                item.quality_index = Some(qi);
                                // CRITICAL FIX: Change status to indicate extraction is complete and ready for download
                                // This prevents items from getting stuck in Extracting state forever
                                item.status = BatchItemStatus::ReadyForDownload; // Ready for download processing
                                tracing::debug!("Item {} extraction completed, ready for download (quality index: {})", item_id, qi);
                            } else {
                                // No valid qualities available - provide detailed error message
                                let error_message = if video_info.qualities.is_empty() {
                                    format!("No downloadable video streams found. Video metadata was extracted successfully (Title: '{}', Author: '{}'), but no video URLs could be found. This usually indicates: 1) The video is private or requires authentication, 2) Geographic restrictions apply, 3) Facebook is blocking automated access, or 4) The video format is not supported.",
                                        video_info.title,
                                        video_info.metadata.author)
                                } else {
                                    format!("Found {} video qualities but none are downloadable. This may indicate corrupted or inaccessible video streams.", video_info.qualities.len())
                                };

                                item.mark_failed(error_message);
                                tracing::warn!("Item {} marked as failed: no valid qualities available (total qualities: {}, content_type: {:?}, privacy_level: {:?})",
                                    item_id, video_info.qualities.len(), video_info.content_type, video_info.privacy_level);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Extraction failed for item {}: {}", item_id, e);
                // Mark item as failed
                let mut active_batches = self.active_batches.write().await;
                if let Some(batch) = active_batches.get_mut(&batch_id) {
                    if let Some(item) = batch.items.iter_mut().find(|i| i.id == item_id) {
                        item.mark_failed(format!("Extraction failed: {}", e));
                    }
                }
            }
        }
    }

    /// Start download for a specific item
    async fn start_item_download(&self, batch_id: &str, item_id: &str) -> Result<()> {
        // Check if item is already being downloaded or completed
        {
            let active_batches = self.active_batches.read().await;
            if let Some(batch) = active_batches.get(batch_id) {
                if let Some(item) = batch.items.iter().find(|i| i.id == item_id) {
                    match item.status {
                        BatchItemStatus::Downloading | BatchItemStatus::Compressing | BatchItemStatus::Completed => {
                            tracing::debug!("Item {} already in progress or completed, skipping download", item_id);
                            return Ok(());
                        }
                        BatchItemStatus::ReadyForDownload => {
                            // This is the expected state, continue
                        }
                        _ => {
                            tracing::warn!("Item {} not ready for download (status: {:?})", item_id, item.status);
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Get item info for download
        let (video_info, quality_index, output_dir) = {
            let active_batches = self.active_batches.read().await;
            if let Some(batch) = active_batches.get(batch_id) {
                if let Some(item) = batch.items.iter().find(|i| i.id == item_id) {
                    if let (Some(video_info), Some(quality_index)) = (&item.video_info, item.quality_index) {
                        (video_info.clone(), quality_index, batch.options.output_directory.clone())
                    } else {
                        tracing::warn!("Item {} missing video_info or quality_index", item_id);
                        return Ok(());
                    }
                } else {
                    tracing::warn!("Item {} not found in batch {}", item_id, batch_id);
                    return Ok(());
                }
            } else {
                tracing::warn!("Batch {} not found", batch_id);
                return Ok(());
            }
        };

        // Update item status to downloading
        {
            let mut active_batches = self.active_batches.write().await;
            if let Some(batch) = active_batches.get_mut(batch_id) {
                if let Some(item) = batch.items.iter_mut().find(|i| i.id == item_id) {
                    tracing::info!("Starting download for item {} ({})", item_id, video_info.title);
                    item.status = BatchItemStatus::Downloading;
                }
            }
        }

        // Create download task
        let output_path = if let Some(dir) = output_dir {
            format!("{}/{}.mp4", dir, video_info.video_id)
        } else {
            format!("{}.mp4", video_info.video_id)
        };

        let download_task = DownloadTask::new(
            item_id.to_string(),
            video_info.clone(),
            quality_index,
            output_path.clone(),
        );

        // Add to download manager (handle duplicate task errors)
        match self.download_manager.add_task(download_task).await {
            Ok(()) => {
                tracing::debug!("Added download task for item {}", item_id);
            }
            Err(e) if e.to_string().contains("already exists") => {
                tracing::warn!("Download task for item {} already exists, skipping", item_id);
                return Ok(());
            }
            Err(e) => {
                tracing::error!("Failed to add download task for item {}: {}", item_id, e);
                // Mark item as failed
                let mut active_batches = self.active_batches.write().await;
                if let Some(batch) = active_batches.get_mut(batch_id) {
                    if let Some(item) = batch.items.iter_mut().find(|i| i.id == item_id) {
                        item.mark_failed(format!("Failed to add download task: {}", e));
                    }
                }
                return Err(e);
            }
        }

        // Check download manager state before starting
        let active_count = self.download_manager.active_download_count().await;
        let can_start = self.download_manager.can_start_download().await;
        tracing::info!("Download manager state: {} active downloads, can start: {}", active_count, can_start);

        // Start the download (handle duplicate start errors)
        match self.download_manager.start_download(&item_id, None).await {
            Ok(()) => {
                tracing::info!("Successfully started download for item {} ({})", item_id, video_info.title);
            }
            Err(e) if e.to_string().contains("not in queued state") => {
                tracing::warn!("Download for item {} already started, skipping", item_id);
                return Ok(());
            }
            Err(e) => {
                tracing::error!("Failed to start download for item {} ({}): {}", item_id, video_info.title, e);
                // Mark item as failed
                let mut active_batches = self.active_batches.write().await;
                if let Some(batch) = active_batches.get_mut(batch_id) {
                    if let Some(item) = batch.items.iter_mut().find(|i| i.id == item_id) {
                        item.mark_failed(format!("Failed to start download: {}", e));
                    }
                }
                return Err(e);
            }
        }

        // Spawn monitoring task for this download with a small delay to ensure the download task is properly initialized
        let processor = self.clone();
        let batch_id = batch_id.to_string();
        let item_id = item_id.to_string();

        tokio::spawn(async move {
            // Small delay to ensure download task is properly initialized
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            processor.monitor_download_progress(batch_id, item_id).await;
        });

        Ok(())
    }

    /// Monitor download progress for an item
    async fn monitor_download_progress(&self, batch_id: String, item_id: String) {
        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 5; // Allow 5 seconds for the download task to appear

        loop {
            // Check if download is still active
            let download_task = self.download_manager.get_task(&item_id).await;

            if let Some(task) = download_task {
                // Reset retry count since we found the task
                retry_count = 0;

                // Update item progress and batch progress
                {
                    let mut active_batches = self.active_batches.write().await;
                    if let Some(batch) = active_batches.get_mut(&batch_id) {
                        if let Some(item) = batch.items.iter_mut().find(|i| i.id == item_id) {
                            item.download_progress = Some(task.progress.clone());

                            // Check if download completed
                            match task.status {
                                DownloadStatus::Completed => {
                                    tracing::info!("Download completed for item {} ({})", item_id, item.url);
                                    // Check if compression is enabled
                                    let should_compress = batch.options.enable_compression;
                                    if should_compress {
                                        item.status = BatchItemStatus::Compressing;
                                        // Start compression
                                        let processor = self.clone();
                                        let batch_id_clone = batch_id.clone();
                                        let item_id_clone = item_id.clone();
                                        let output_path = task.output_path.clone();
                                        tokio::spawn(async move {
                                            processor.start_item_compression(batch_id_clone, item_id_clone, output_path).await;
                                        });
                                    } else {
                                        item.mark_completed(Some(task.output_path));
                                    }
                                    break;
                                }
                                DownloadStatus::Failed => {
                                    let error_msg = task.progress.error_message
                                        .unwrap_or_else(|| "Download failed".to_string());
                                    tracing::error!("Download failed for item {} ({}): {}", item_id, item.url, error_msg);
                                    item.mark_failed(error_msg);
                                    break;
                                }
                                DownloadStatus::Cancelled => {
                                    tracing::info!("Download cancelled for item {} ({})", item_id, item.url);
                                    item.status = BatchItemStatus::Cancelled;
                                    item.completed_at = Some(Utc::now());
                                    break;
                                }
                                _ => {
                                    // Still in progress, continue monitoring
                                    tracing::debug!("Download in progress for item {} ({}): {:?}", item_id, item.url, task.status);
                                }
                            }
                        }
                        // Update batch progress after any item status change
                        batch.update_progress();
                    }
                }
            } else {
                // Download task not found - this might be a race condition
                retry_count += 1;

                if retry_count <= MAX_RETRIES {
                    tracing::debug!("Download task not found for item {} (attempt {}/{}), retrying...", item_id, retry_count, MAX_RETRIES);
                    // Wait a bit longer for the task to appear
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    continue;
                } else {
                    // Task really doesn't exist, mark as failed
                    tracing::error!("Download task not found for item {} after {} retries", item_id, MAX_RETRIES);
                    let mut active_batches = self.active_batches.write().await;
                    if let Some(batch) = active_batches.get_mut(&batch_id) {
                        if let Some(item) = batch.items.iter_mut().find(|i| i.id == item_id) {
                            item.mark_failed("Download task not found after retries".to_string());
                        }
                        batch.update_progress();
                    }
                    break;
                }
            }

            // Wait before next check
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        }
    }

    /// Start compression for a specific item
    async fn start_item_compression(&self, batch_id: String, item_id: String, input_path: String) {
        // Check if compression is available
        if !self.compression_service.is_available() {
            tracing::warn!("Compression requested but FFmpeg not available, skipping compression for item {}", item_id);
            // Mark as completed with original file
            let mut active_batches = self.active_batches.write().await;
            if let Some(batch) = active_batches.get_mut(&batch_id) {
                if let Some(item) = batch.items.iter_mut().find(|i| i.id == item_id) {
                    item.mark_completed(Some(input_path));
                }
                batch.update_progress();
            }
            return;
        }

        // Get compression options
        let (compression_quality, preserve_original) = {
            let active_batches = self.active_batches.read().await;
            if let Some(batch) = active_batches.get(&batch_id) {
                (
                    batch.options.compression_quality,
                    batch.options.preserve_original_on_compression,
                )
            } else {
                return;
            }
        };

        // Create output path for compressed file
        let output_path = if preserve_original {
            let path = std::path::Path::new(&input_path);
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            let extension = path.extension().unwrap_or_default().to_string_lossy();
            let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
            format!("{}/{}_{}.{}", parent.display(), stem, compression_quality.label(), extension)
        } else {
            input_path.clone()
        };

        let compression_options = CompressionOptions {
            quality: compression_quality,
            input_path: input_path.clone().into(),
            output_path: output_path.clone().into(),
            preserve_original,
            codec: "libx264".to_string(),
            preset: "medium".to_string(),
        };

        // Perform compression
        match self.compression_service.compress_video(compression_options, None).await {
            Ok(result) => {
                // Update item with compression result
                let mut active_batches = self.active_batches.write().await;
                if let Some(batch) = active_batches.get_mut(&batch_id) {
                    if let Some(item) = batch.items.iter_mut().find(|i| i.id == item_id) {
                        item.compression_result = Some(result);
                        item.mark_completed(Some(output_path));
                    }
                    batch.update_progress();
                }
            }
            Err(e) => {
                // Mark as failed but keep the original download
                let mut active_batches = self.active_batches.write().await;
                if let Some(batch) = active_batches.get_mut(&batch_id) {
                    if let Some(item) = batch.items.iter_mut().find(|i| i.id == item_id) {
                        item.mark_failed(format!("Compression failed: {}", e));
                        // Still provide the original file path
                        item.output_path = Some(input_path);
                    }
                    batch.update_progress();
                }
            }
        }
    }

    /// Update batch progress
    async fn update_batch_progress(&self, batch_id: &str) -> Result<()> {
        let mut active_batches = self.active_batches.write().await;
        if let Some(batch) = active_batches.get_mut(batch_id) {
            batch.update_progress();
        }
        Ok(())
    }

    /// Finalize a completed batch
    async fn finalize_batch(&self, batch_id: &str) -> Result<()> {
        let mut active_batches = self.active_batches.write().await;
        if let Some(batch) = active_batches.get_mut(batch_id) {
            batch.completed_at = Some(Utc::now());
            batch.progress.completed_at = batch.completed_at;
            batch.update_progress();

            tracing::info!(
                "Batch '{}' completed: {}/{} items successful",
                batch.name,
                batch.progress.completed_items,
                batch.progress.total_items
            );
        }
        Ok(())
    }
}
