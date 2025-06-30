//! Download functionality and progress tracking

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::AsyncWriteExt;
use std::path::Path;
use std::time::Instant;
use tracing::{debug, error, info, warn};
use futures_util::StreamExt;
use crate::{
    common::{Result, FacebookExtractorError, types::VideoInfo},
    network::anti_blocking::{AntiBlockingManager, AntiBlockingConfig, AntiBlockingStats}
};

/// Download status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    /// Download is queued but not started
    Queued,
    /// Download is starting
    Starting,
    /// Download is in progress
    Downloading,
    /// Download is paused
    Paused,
    /// Download completed successfully
    Completed,
    /// Download failed
    Failed,
    /// Download was cancelled
    Cancelled,
}

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub id: String,
    pub progress: f64, // 0.0 to 100.0
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed_bytes_per_sec: Option<u64>,
    pub eta_seconds: Option<u64>,
    pub status: DownloadStatus,
    pub error_message: Option<String>,
}

/// Download task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub video_info: VideoInfo,
    pub quality_index: usize,
    pub output_path: String,
    pub status: DownloadStatus,
    pub progress: DownloadProgress,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl DownloadTask {
    /// Create a new download task
    pub fn new(
        id: String,
        video_info: VideoInfo,
        quality_index: usize,
        output_path: String,
    ) -> Self {
        let progress = DownloadProgress {
            id: id.clone(),
            progress: 0.0,
            downloaded_bytes: 0,
            total_bytes: None,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            status: DownloadStatus::Queued,
            error_message: None,
        };

        Self {
            id,
            video_info,
            quality_index,
            output_path,
            status: DownloadStatus::Queued,
            progress,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Update download progress
    pub fn update_progress(&mut self, progress: DownloadProgress) {
        self.status = progress.status.clone();
        self.progress = progress;

        // Update timestamps based on status
        match self.status {
            DownloadStatus::Starting | DownloadStatus::Downloading => {
                if self.started_at.is_none() {
                    self.started_at = Some(chrono::Utc::now());
                }
            }
            DownloadStatus::Completed | DownloadStatus::Failed | DownloadStatus::Cancelled => {
                if self.completed_at.is_none() {
                    self.completed_at = Some(chrono::Utc::now());
                }
            }
            _ => {}
        }
    }

    /// Get download duration
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end.signed_duration_since(start)),
            (Some(start), None) => Some(chrono::Utc::now().signed_duration_since(start)),
            _ => None,
        }
    }

    /// Check if download is active
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            DownloadStatus::Starting | DownloadStatus::Downloading
        )
    }

    /// Check if download is finished
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            DownloadStatus::Completed | DownloadStatus::Failed | DownloadStatus::Cancelled
        )
    }
}

/// Download manager for handling multiple concurrent downloads
#[derive(Debug)]
pub struct DownloadManager {
    tasks: Arc<Mutex<Vec<DownloadTask>>>,
    max_concurrent: usize,
    anti_blocking_manager: Option<Arc<AntiBlockingManager>>,
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            max_concurrent,
            anti_blocking_manager: None,
        }
    }

    /// Create a new download manager with anti-blocking capabilities
    pub fn new_with_anti_blocking(max_concurrent: usize, anti_blocking_config: AntiBlockingConfig) -> Self {
        let anti_blocking_manager = AntiBlockingManager::new(anti_blocking_config);
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            max_concurrent,
            anti_blocking_manager: Some(Arc::new(anti_blocking_manager)),
        }
    }

    /// Initialize the download manager (required when using anti-blocking)
    pub async fn initialize(&self) -> Result<()> {
        if let Some(ref manager) = self.anti_blocking_manager {
            manager.initialize().await?;
            info!("✅ Download manager initialized with anti-blocking capabilities");
        }
        Ok(())
    }

    /// Add a new download task
    pub async fn add_task(&self, task: DownloadTask) -> Result<()> {
        let mut tasks = self.tasks.lock().await;

        // Check if a task with this ID already exists
        if tasks.iter().any(|t| t.id == task.id) {
            return Err(FacebookExtractorError::download(
                format!("Task with ID '{}' already exists", task.id)
            ));
        }

        tasks.push(task);
        Ok(())
    }

    /// Get all download tasks
    pub async fn get_tasks(&self) -> Vec<DownloadTask> {
        let tasks = self.tasks.lock().await;
        tasks.clone()
    }

    /// Get a specific download task by ID
    pub async fn get_task(&self, id: &str) -> Option<DownloadTask> {
        let tasks = self.tasks.lock().await;
        tasks.iter().find(|task| task.id == id).cloned()
    }

    /// Update download progress for a specific task
    pub async fn update_progress(&self, id: &str, progress: DownloadProgress) -> Result<()> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.iter_mut().find(|task| task.id == id) {
            task.update_progress(progress);
            Ok(())
        } else {
            Err(FacebookExtractorError::download(format!(
                "Download task not found: {}",
                id
            )))
        }
    }

    /// Cancel a download task
    pub async fn cancel_task(&self, id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.iter_mut().find(|task| task.id == id) {
            if task.is_active() {
                task.status = DownloadStatus::Cancelled;
                task.progress.status = DownloadStatus::Cancelled;
                task.completed_at = Some(chrono::Utc::now());
            }
            Ok(())
        } else {
            Err(FacebookExtractorError::download(format!(
                "Download task not found: {}",
                id
            )))
        }
    }

    /// Remove completed/failed/cancelled tasks
    pub async fn cleanup_finished_tasks(&self) {
        let mut tasks = self.tasks.lock().await;
        tasks.retain(|task| !task.is_finished());
    }

    /// Clean up stuck downloads that have been in Starting/Downloading state for too long
    pub async fn cleanup_stuck_downloads(&self) {
        let mut tasks = self.tasks.lock().await;
        let now = chrono::Utc::now();
        let timeout_duration = chrono::Duration::minutes(5); // 5 minute timeout

        for task in tasks.iter_mut() {
            if task.is_active() {
                let duration_since_created = now.signed_duration_since(task.created_at);
                if duration_since_created > timeout_duration {
                    tracing::warn!("🧹 Cleaning up stuck download task: {} (stuck for {} minutes)",
                        task.id, duration_since_created.num_minutes());
                    task.status = DownloadStatus::Failed;
                    task.progress.status = DownloadStatus::Failed;
                    task.progress.error_message = Some("Download timed out".to_string());
                    task.completed_at = Some(now);
                }
            }
        }
    }

    /// Get active download count
    pub async fn active_download_count(&self) -> usize {
        let tasks = self.tasks.lock().await;
        tasks.iter().filter(|task| task.is_active()).count()
    }

    /// Check if we can start a new download
    pub async fn can_start_download(&self) -> bool {
        self.active_download_count().await < self.max_concurrent
    }

    /// Start downloading a specific task
    pub async fn start_download(&self, task_id: &str, progress_callback: Option<ProgressCallback>) -> Result<()> {
        tracing::info!("🚀 Starting download for task: {}", task_id);

        // Get the task
        let task = {
            let mut tasks = self.tasks.lock().await;
            tracing::debug!("📊 Current tasks in download manager: {}", tasks.len());

            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                tracing::debug!("✅ Found task {} with status: {:?}", task_id, task.status);
                if task.status != DownloadStatus::Queued {
                    tracing::warn!("❌ Task {} is not in queued state: {:?}", task_id, task.status);
                    return Err(FacebookExtractorError::download(
                        format!("Task {} is not in queued state", task_id)
                    ));
                }
                task.status = DownloadStatus::Starting;
                task.progress.status = DownloadStatus::Starting;
                task.started_at = Some(chrono::Utc::now());
                tracing::info!("🔄 Task {} status updated to Starting", task_id);
                task.clone()
            } else {
                tracing::error!("❌ Task not found: {}", task_id);
                return Err(FacebookExtractorError::download(
                    format!("Task not found: {}", task_id)
                ));
            }
        };

        // Check if we can start the download
        let active_count = self.active_download_count().await;
        let can_start = self.can_start_download().await;
        tracing::info!("📊 Download capacity check: {} active, max: {}, can start: {}", active_count, self.max_concurrent, can_start);

        if !can_start {
            tracing::warn!("❌ Maximum concurrent downloads reached: {}/{}", active_count, self.max_concurrent);
            return Err(FacebookExtractorError::download(
                "Maximum concurrent downloads reached"
            ));
        }

        // Get the video quality to download
        let quality = task.video_info.qualities.get(task.quality_index)
            .ok_or_else(|| FacebookExtractorError::download(
                format!("Invalid quality index: {}", task.quality_index)
            ))?;

        // Clone necessary data for the async task
        let task_id = task.id.clone();
        let url = quality.download_url.clone();
        let output_path = task.output_path.clone();
        let tasks_ref = self.tasks.clone();

        // Clone anti-blocking manager for the async task
        let anti_blocking_manager = self.anti_blocking_manager.clone();

        // Spawn the download task
        tracing::info!("🚀 Spawning download task for {} -> {}", task_id, output_path);
        tokio::spawn(async move {
            tracing::info!("📥 Download task started for {}", task_id);
            let result = Self::download_file(&url, &output_path, &task_id, tasks_ref.clone(), progress_callback, anti_blocking_manager).await;

            // Update task status based on result
            let mut tasks = tasks_ref.lock().await;
            if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
                match result {
                    Ok(_) => {
                        tracing::info!("✅ Download completed for {}", task_id);
                        task.status = DownloadStatus::Completed;
                        task.progress.status = DownloadStatus::Completed;
                        task.progress.progress = 100.0;
                        task.completed_at = Some(chrono::Utc::now());
                    }
                    Err(e) => {
                        tracing::error!("❌ Download failed for {}: {}", task_id, e);
                        task.status = DownloadStatus::Failed;
                        task.progress.status = DownloadStatus::Failed;
                        task.progress.error_message = Some(e.to_string());
                        task.completed_at = Some(chrono::Utc::now());
                    }
                }
            }
        });

        Ok(())
    }

    /// Internal method to download a file
    async fn download_file(
        url: &str,
        output_path: &str,
        task_id: &str,
        tasks: Arc<Mutex<Vec<DownloadTask>>>,
        progress_callback: Option<ProgressCallback>,
        anti_blocking_manager: Option<Arc<AntiBlockingManager>>,
    ) -> Result<()> {
        // Create output directory if it doesn't exist
        if let Some(parent) = Path::new(output_path).parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| FacebookExtractorError::download(
                    format!("Failed to create output directory: {}", e)
                ))?;
        }

        // Start the download with anti-blocking if available
        let response = if let Some(ref manager) = anti_blocking_manager {
            info!("🛡️ Using anti-blocking manager for download request");
            manager.make_request(url).await?
        } else {
            info!("📡 Using standard HTTP client for download request");
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| FacebookExtractorError::download(
                    format!("Failed to create HTTP client: {}", e)
                ))?;

            let response = client.get(url).send().await
                .map_err(|e| FacebookExtractorError::download(
                    format!("Failed to start download: {}", e)
                ))?;

            if !response.status().is_success() {
                return Err(FacebookExtractorError::download(
                    format!("HTTP error: {}", response.status())
                ));
            }

            response
        };

        let total_size = response.content_length();
        let mut file = tokio::fs::File::create(output_path).await
            .map_err(|e| FacebookExtractorError::download(
                format!("Failed to create output file: {}", e)
            ))?;

        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        let start_time = Instant::now();

        // Update status to downloading
        {
            let mut tasks_guard = tasks.lock().await;
            if let Some(task) = tasks_guard.iter_mut().find(|t| t.id == task_id) {
                task.status = DownloadStatus::Downloading;
                task.progress.status = DownloadStatus::Downloading;
                task.progress.total_bytes = total_size;
            }
        }

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| FacebookExtractorError::download(
                format!("Download stream error: {}", e)
            ))?;

            file.write_all(&chunk).await
                .map_err(|e| FacebookExtractorError::download(
                    format!("Failed to write to file: {}", e)
                ))?;

            downloaded += chunk.len() as u64;

            // Calculate progress and speed
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { downloaded as f64 / elapsed } else { 0.0 };
            let progress_percentage = if let Some(total) = total_size {
                (downloaded as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            let eta = if speed > 0.0 && total_size.is_some() {
                let remaining = total_size.unwrap() - downloaded;
                Some((remaining as f64 / speed) as u64)
            } else {
                None
            };

            // Update progress
            let progress = DownloadProgress {
                id: task_id.to_string(),
                progress: progress_percentage,
                downloaded_bytes: downloaded,
                total_bytes: total_size,
                speed_bytes_per_sec: Some(speed as u64),
                eta_seconds: eta,
                status: DownloadStatus::Downloading,
                error_message: None,
            };

            // Update task progress
            {
                let mut tasks_guard = tasks.lock().await;
                if let Some(task) = tasks_guard.iter_mut().find(|t| t.id == task_id) {
                    task.progress = progress.clone();
                }
            }

            // Call progress callback
            if let Some(ref callback) = progress_callback {
                callback(progress);
            }
        }

        file.flush().await
            .map_err(|e| FacebookExtractorError::download(
                format!("Failed to flush file: {}", e)
            ))?;

        Ok(())
    }

    /// Get download statistics
    pub async fn get_statistics(&self) -> DownloadStatistics {
        let tasks = self.tasks.lock().await;

        let total_tasks = tasks.len();
        let completed = tasks.iter().filter(|t| t.status == DownloadStatus::Completed).count();
        let failed = tasks.iter().filter(|t| t.status == DownloadStatus::Failed).count();
        let active = tasks.iter().filter(|t| t.is_active()).count();
        let queued = tasks.iter().filter(|t| t.status == DownloadStatus::Queued).count();

        let total_downloaded_bytes = tasks
            .iter()
            .map(|t| t.progress.downloaded_bytes)
            .sum();

        let average_speed = if active > 0 {
            let total_speed: u64 = tasks
                .iter()
                .filter(|t| t.is_active())
                .filter_map(|t| t.progress.speed_bytes_per_sec)
                .sum();
            Some(total_speed / active as u64)
        } else {
            None
        };

        DownloadStatistics {
            total_tasks,
            completed,
            failed,
            active,
            queued,
            total_downloaded_bytes,
            average_speed_bytes_per_sec: average_speed,
        }
    }

    /// Cleanup anti-blocking resources
    pub async fn cleanup(&self) -> Result<()> {
        if let Some(ref manager) = self.anti_blocking_manager {
            manager.cleanup().await?;
            info!("✅ Download manager anti-blocking cleanup completed");
        }
        Ok(())
    }

    /// Get anti-blocking statistics
    pub async fn get_anti_blocking_stats(&self) -> Option<AntiBlockingStats> {
        if let Some(ref manager) = self.anti_blocking_manager {
            Some(manager.get_stats().await)
        } else {
            None
        }
    }
}

/// Download statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStatistics {
    pub total_tasks: usize,
    pub completed: usize,
    pub failed: usize,
    pub active: usize,
    pub queued: usize,
    pub total_downloaded_bytes: u64,
    pub average_speed_bytes_per_sec: Option<u64>,
}

/// Progress callback type for download operations
pub type ProgressCallback = Box<dyn Fn(DownloadProgress) + Send + Sync>;

/// Download options
#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub output_path: String,
    pub quality_index: usize,
    pub enable_resume: bool,
    pub chunk_size_bytes: usize,
    pub max_retries: usize,
    pub verify_download: bool,
}
