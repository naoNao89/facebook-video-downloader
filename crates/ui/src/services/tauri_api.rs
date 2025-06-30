//! Tauri API service for communicating with the backend

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::components::compression::{CompressionQuality, CompressionEstimate};
use serde_wasm_bindgen;
use web_sys::console;
use wasm_bindgen_futures;

// Import Tauri API for v2 - using dynamic approach
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "eval")]
    fn js_eval(code: &str) -> JsValue;
}

/// Check if we're running in a Tauri environment (v2 compatible)
pub fn is_tauri_available() -> bool {
    console::log_1(&format!("🔍 Tauri API Detection Debug:").into());

    // Check if window object exists
    let window = match web_sys::window() {
        Some(w) => {
            console::log_1(&format!("  - Window object: available").into());
            w
        }
        None => {
            console::log_1(&format!("  - Window object: NOT available").into());
            return false;
        }
    };

    // Check if window.__TAURI__ exists
    let tauri_obj = js_sys::Reflect::get(&window, &"__TAURI__".into()).unwrap_or(JsValue::UNDEFINED);
    let tauri_available = !tauri_obj.is_undefined();
    console::log_1(&format!("  - window.__TAURI__: {}", if tauri_available { "available" } else { "undefined" }).into());

    // Check for alternative Tauri objects (v2 uses __TAURI_INTERNALS__)
    let tauri_internals = js_sys::Reflect::get(&window, &"__TAURI_INTERNALS__".into()).unwrap_or(JsValue::UNDEFINED);
    let internals_available = !tauri_internals.is_undefined();
    console::log_1(&format!("  - window.__TAURI_INTERNALS__: {}", if internals_available { "available" } else { "undefined" }).into());

    // List all window properties for debugging
    let keys = js_sys::Object::keys(&window);
    let mut tauri_keys = Vec::new();
    for i in 0..keys.length() {
        if let Some(key) = keys.get(i).as_string() {
            if key.contains("TAURI") || key.contains("tauri") {
                tauri_keys.push(key);
            }
        }
    }
    console::log_1(&format!("  - Tauri-related window properties: {:?}", tauri_keys).into());

    // In Tauri v2, we need either __TAURI__ or __TAURI_INTERNALS__
    if !tauri_available && !internals_available {
        console::log_1(&format!("  - Neither __TAURI__ nor __TAURI_INTERNALS__ found").into());
        return false;
    }

    // If we have __TAURI_INTERNALS__, check for invoke function
    if internals_available {
        let invoke_obj = js_sys::Reflect::get(&tauri_internals, &"invoke".into()).unwrap_or(JsValue::UNDEFINED);
        let invoke_available = !invoke_obj.is_undefined();
        console::log_1(&format!("  - window.__TAURI_INTERNALS__.invoke: {}", if invoke_available { "available" } else { "undefined" }).into());

        if invoke_available {
            console::log_1(&format!("  - Overall result: true (using __TAURI_INTERNALS__)").into());
            return true;
        }
    }

    // If we have __TAURI__, check for invoke function (legacy or alternative structure)
    if tauri_available {
        // Try direct invoke
        let invoke_obj = js_sys::Reflect::get(&tauri_obj, &"invoke".into()).unwrap_or(JsValue::UNDEFINED);
        let invoke_available = !invoke_obj.is_undefined();
        console::log_1(&format!("  - window.__TAURI__.invoke: {}", if invoke_available { "available" } else { "undefined" }).into());

        if invoke_available {
            console::log_1(&format!("  - Overall result: true (using __TAURI__.invoke)").into());
            return true;
        }

        // Try core.invoke (v1 style)
        let core_obj = js_sys::Reflect::get(&tauri_obj, &"core".into()).unwrap_or(JsValue::UNDEFINED);
        let core_available = !core_obj.is_undefined();
        console::log_1(&format!("  - window.__TAURI__.core: {}", if core_available { "available" } else { "undefined" }).into());

        if core_available {
            let core_invoke_obj = js_sys::Reflect::get(&core_obj, &"invoke".into()).unwrap_or(JsValue::UNDEFINED);
            let core_invoke_available = !core_invoke_obj.is_undefined();
            console::log_1(&format!("  - window.__TAURI__.core.invoke: {}", if core_invoke_available { "available" } else { "undefined" }).into());

            if core_invoke_available {
                console::log_1(&format!("  - Overall result: true (using __TAURI__.core.invoke)").into());
                return true;
            }
        }

        // List properties of __TAURI__ object for debugging
        let tauri_keys = js_sys::Object::keys(&tauri_obj.into());
        let mut keys_list = Vec::new();
        for i in 0..tauri_keys.length() {
            if let Some(key) = tauri_keys.get(i).as_string() {
                keys_list.push(key);
            }
        }
        console::log_1(&format!("  - window.__TAURI__ properties: {:?}", keys_list).into());
    }

    console::log_1(&format!("  - Overall result: false").into());
    false
}

/// Dynamic invoke function that works with different Tauri API structures
async fn dynamic_invoke(command: &str, args: JsValue) -> Result<JsValue, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("Window not available"))?;

    // Try different Tauri API structures

    // 1. Try __TAURI_INTERNALS__.invoke (Tauri v2)
    if let Ok(tauri_internals) = js_sys::Reflect::get(&window, &"__TAURI_INTERNALS__".into()) {
        if !tauri_internals.is_undefined() {
            if let Ok(invoke_fn) = js_sys::Reflect::get(&tauri_internals, &"invoke".into()) {
                if !invoke_fn.is_undefined() {
                    console::log_1(&"Using __TAURI_INTERNALS__.invoke".into());
                    let invoke_fn = invoke_fn.dyn_into::<js_sys::Function>()
                        .map_err(|_| JsValue::from_str("invoke is not a function"))?;

                    let promise = invoke_fn.call2(&tauri_internals, &command.into(), &args)
                        .map_err(|e| e)?;

                    return wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise))
                        .await;
                }
            }
        }
    }

    // 2. Try __TAURI__.invoke (Tauri v2 alternative)
    if let Ok(tauri_obj) = js_sys::Reflect::get(&window, &"__TAURI__".into()) {
        if !tauri_obj.is_undefined() {
            if let Ok(invoke_fn) = js_sys::Reflect::get(&tauri_obj, &"invoke".into()) {
                if !invoke_fn.is_undefined() {
                    console::log_1(&"Using __TAURI__.invoke".into());
                    let invoke_fn = invoke_fn.dyn_into::<js_sys::Function>()
                        .map_err(|_| JsValue::from_str("invoke is not a function"))?;

                    let promise = invoke_fn.call2(&tauri_obj, &command.into(), &args)
                        .map_err(|e| e)?;

                    return wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise))
                        .await;
                }
            }

            // 3. Try __TAURI__.core.invoke (Tauri v1 style)
            if let Ok(core_obj) = js_sys::Reflect::get(&tauri_obj, &"core".into()) {
                if !core_obj.is_undefined() {
                    if let Ok(invoke_fn) = js_sys::Reflect::get(&core_obj, &"invoke".into()) {
                        if !invoke_fn.is_undefined() {
                            console::log_1(&"Using __TAURI__.core.invoke".into());
                            let invoke_fn = invoke_fn.dyn_into::<js_sys::Function>()
                                .map_err(|_| JsValue::from_str("invoke is not a function"))?;

                            let promise = invoke_fn.call2(&core_obj, &command.into(), &args)
                                .map_err(|e| e)?;

                            return wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise))
                                .await;
                        }
                    }
                }
            }
        }
    }

    Err(JsValue::from_str("No Tauri invoke function found"))
}

/// Generic result type for Tauri commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriResult<T> {
    pub data: T,
    pub message: Option<String>,
}

/// Error response from Tauri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriError {
    pub error: String,
    pub details: Option<String>,
    pub code: Option<String>,
    pub recoverable: bool,
}

/// Video information from extraction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoInfo {
    pub title: String,
    pub duration: String,
    pub thumbnail: String,
    pub qualities: Vec<VideoQuality>,
    pub video_id: String,
    pub metadata: VideoMetadata,
    pub extraction_timestamp: String,
    pub source_url: String,
    pub content_type: String,
    pub privacy_level: String,
    pub access_method: String,
}

/// Stream type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamType {
    CompleteVideoAudio,
    VideoOnly,
    AudioOnly,
    CombinedVideoAudio,
    Unknown,
}

/// Video quality information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoQuality {
    pub quality: String,
    pub size: String,
    pub format: String,
    pub download_url: String,
    pub width: u32,
    pub height: u32,
    pub stream_type: StreamType,
    pub efg_metadata: String,
    pub estimated_size_mb: u32,
    pub bitrate_kbps: Option<u32>,
    pub fps: Option<u32>,
    pub codec: Option<String>,
}

/// Video metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub author: String,
    pub description: String,
    pub publish_date: String,
    pub likes: u64,
    pub comments: u64,
    pub views: u64,
    pub shares: u64,
    pub hashtags: Vec<String>,
    pub duration_seconds: Option<u32>,
    pub language: Option<String>,
    pub category: Option<String>,
    pub author_url: Option<String>,
    pub author_verified: bool,
    pub privacy_level: Option<String>,
    pub location: Option<String>,
    pub content_warnings: Vec<String>,
}

/// URL validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlValidation {
    pub is_valid: bool,
    pub content_type: Option<String>,
    pub video_id: Option<String>,
    pub error_message: Option<String>,
    pub suggestions: Vec<String>,
}

/// Download request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub video_info: VideoInfo,
    pub quality_index: usize,
    pub output_path: Option<String>,
    pub custom_filename: Option<String>,
}

/// Download progress update
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub id: String,
    pub progress: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed_bytes_per_sec: Option<u64>,
    pub eta_seconds: Option<u64>,
    pub status: String,
    pub error_message: Option<String>,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub download_dir: String,
    pub max_concurrent_downloads: usize,
    pub auto_open_downloads: bool,
    pub show_notifications: bool,
    pub theme: String,
    pub language: String,
    pub quality_preference: String,
    pub filename_template: String,
    pub enable_ffmpeg: bool,
    pub ffmpeg_path: Option<String>,
    pub network_timeout_secs: u64,
    pub max_retries: usize,
    pub enable_view_source: bool,
    pub respect_privacy: bool,
    pub enable_debug_logging: bool,
}

/// Helper function to call Tauri commands
async fn call_tauri<T, R>(command: &str, args: T) -> Result<TauriResult<R>, TauriError>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    console::log_1(&format!("🚀 Attempting to call Tauri command: {}", command).into());

    // Check if Tauri API is available
    if !is_tauri_available() {
        console::error_1(&"❌ Tauri API not available! Make sure you're running the app with 'tauri dev' or 'tauri build', not 'trunk serve'.".into());
        return Err(TauriError {
            error: "Tauri API not available. Please run the application using 'pnpm run tauri:dev' instead of 'pnpm run dev'.".to_string(),
            details: Some("The frontend is running in standalone mode without the Tauri backend. Use 'tauri dev' to run both frontend and backend together.".to_string()),
            code: Some("TAURI_API_UNAVAILABLE".to_string()),
            recoverable: true,
        });
    }

    console::log_1(&"✅ Tauri API is available - proceeding with command".into());

    let args_value = serde_wasm_bindgen::to_value(&args)
        .map_err(|e| TauriError {
            error: format!("Serialization error: {}", e),
            details: None,
            code: None,
            recoverable: false,
        })?;

    // Use dynamic invoke to handle different Tauri API structures
    let result = dynamic_invoke(command, args_value).await
        .map_err(|e| TauriError {
            error: format!("Tauri invoke failed: {:?}", e),
            details: Some("Failed to call Tauri backend command".to_string()),
            code: Some("TAURI_INVOKE_FAILED".to_string()),
            recoverable: true,
        })?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| TauriError {
            error: format!("Deserialization error: {}", e),
            details: None,
            code: None,
            recoverable: false,
        })
}

/// Helper function for commands without arguments
async fn call_tauri_simple<R>(command: &str) -> Result<TauriResult<R>, TauriError>
where
    R: for<'de> Deserialize<'de>,
{
    call_tauri(command, ()).await
}

// Video extraction functions
pub async fn extract_video_info(url: String) -> Result<VideoInfo, TauriError> {
    console::log_1(&format!("🔍 [TAURI_API] Starting extract_video_info for URL: {}", url).into());

    #[derive(Serialize)]
    struct ExtractVideoArgs {
        url: String,
    }

    let args = ExtractVideoArgs { url: url.clone() };

    console::log_1(&"🔍 [TAURI_API] Calling Tauri backend extract_video_info command...".into());

    let result: TauriResult<VideoInfo> = call_tauri("extract_video_info", args).await?;
    console::log_1(&format!("✅ [TAURI_API] Successfully received response from backend for URL: {}", url).into());
    console::log_1(&format!("✅ [TAURI_API] Video title: {}", result.data.title).into());
    console::log_1(&format!("✅ [TAURI_API] Found {} quality options", result.data.qualities.len()).into());
    Ok(result.data)
}



pub async fn validate_facebook_url(url: String) -> Result<UrlValidation, TauriError> {
    #[derive(Serialize)]
    struct ValidateUrlArgs {
        url: String,
    }

    let args = ValidateUrlArgs { url };
    let result: TauriResult<UrlValidation> = call_tauri("validate_facebook_url", args).await?;
    Ok(result.data)
}

// Fetch thumbnail image as base64 data URL to bypass CORS
pub async fn fetch_thumbnail_image(url: String) -> Result<String, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct FetchThumbnailArgs {
        url: String,
    }

    let args = FetchThumbnailArgs { url };
    let result: TauriResult<String> = call_tauri("fetch_thumbnail_image", args).await?;
    Ok(result.data)
}

// Download functions
pub async fn download_video(video_info: VideoInfo, quality_index: usize, output_path: Option<String>) -> Result<String, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct DownloadVideoArgs {
        video_info: VideoInfo,
        quality_index: usize,
        output_path: Option<String>,
    }

    let args = DownloadVideoArgs {
        video_info,
        quality_index,
        output_path,
    };
    let result: TauriResult<String> = call_tauri("download_video", args).await?;
    Ok(result.data)
}

/// Test Tauri connection - simple command to verify API is working
pub async fn test_tauri_connection() -> Result<String, TauriError> {
    let result: TauriResult<String> = call_tauri_simple("test_tauri_connection").await?;
    Ok(result.data)
}

/// Read clipboard content
pub async fn read_clipboard() -> Result<String, TauriError> {
    let result: TauriResult<String> = call_tauri_simple("read_clipboard").await?;
    Ok(result.data)
}

/// Write content to clipboard
pub async fn write_clipboard(text: String) -> Result<String, TauriError> {
    #[derive(Serialize)]
    struct WriteClipboardArgs {
        text: String,
    }

    let args = WriteClipboardArgs { text };
    let result: TauriResult<()> = call_tauri("write_clipboard", args).await?;
    Ok(result.message.unwrap_or_else(|| "Content copied to clipboard".to_string()))
}

// ============================================================================
// COMPRESSION FUNCTIONS
// ============================================================================

/// Check if compression is available (FFmpeg installed)
pub async fn check_compression_availability() -> Result<TauriResult<bool>, TauriError> {
    call_tauri_simple("check_compression_availability").await
}

/// Estimate compressed file size (simple method)
pub async fn estimate_compression_size(
    original_size_mb: u64,
    quality: CompressionQuality,
) -> Result<TauriResult<CompressionEstimate>, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct EstimateCompressionArgs {
        original_size_mb: u64,
        quality: CompressionQuality,
    }

    let args = EstimateCompressionArgs {
        original_size_mb,
        quality,
    };
    call_tauri("estimate_compression_size", args).await
}

/// Estimate compressed file size using actual file analysis
pub async fn estimate_compression_size_from_file(
    file_path: String,
    quality: CompressionQuality,
) -> Result<TauriResult<CompressionEstimate>, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct EstimateCompressionFromFileArgs {
        file_path: String,
        quality: CompressionQuality,
    }

    let args = EstimateCompressionFromFileArgs {
        file_path,
        quality,
    };
    call_tauri("estimate_compression_size_from_file", args).await
}

/// Compress a video file
pub async fn compress_video(
    input_path: String,
    output_path: String,
    quality: CompressionQuality,
    preserve_original: bool,
) -> Result<TauriResult<CompressionResult>, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct CompressVideoArgs {
        input_path: String,
        output_path: String,
        quality: CompressionQuality,
        preserve_original: bool,
    }

    let args = CompressVideoArgs {
        input_path,
        output_path,
        quality,
        preserve_original,
    };
    call_tauri("compress_video", args).await
}

/// Get compression progress
pub async fn get_compression_progress(
    compression_id: String,
) -> Result<TauriResult<Option<CompressionProgress>>, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct GetCompressionProgressArgs {
        compression_id: String,
    }

    let args = GetCompressionProgressArgs { compression_id };
    call_tauri("get_compression_progress", args).await
}

/// Cancel compression
pub async fn cancel_compression(compression_id: String) -> Result<TauriResult<bool>, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct CancelCompressionArgs {
        compression_id: String,
    }

    let args = CancelCompressionArgs { compression_id };
    call_tauri("cancel_compression", args).await
}

// ============================================================================
// BATCH PROCESSING FUNCTIONS
// ============================================================================

/// Start batch processing for multiple URLs
pub async fn start_batch_processing(
    name: String,
    urls: Vec<String>,
    options: Option<BatchOptions>,
) -> Result<TauriResult<String>, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct StartBatchArgs {
        name: String,
        urls: Vec<String>,
        options: Option<BatchOptions>,
    }

    let args = StartBatchArgs { name, urls, options };
    call_tauri("start_batch_processing", args).await
}

/// Get progress for a specific batch
pub async fn get_batch_progress(batch_id: String) -> Result<TauriResult<Option<BatchProgress>>, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct GetBatchProgressArgs {
        batch_id: String,
    }

    let args = GetBatchProgressArgs { batch_id };
    call_tauri("get_batch_progress", args).await
}

/// Get full batch information
pub async fn get_batch_info(batch_id: String) -> Result<TauriResult<Option<BatchJob>>, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct GetBatchInfoArgs {
        batch_id: String,
    }

    let args = GetBatchInfoArgs { batch_id };
    call_tauri("get_batch_info", args).await
}

/// Get all active batches
pub async fn get_all_batches() -> Result<TauriResult<Vec<BatchJob>>, TauriError> {
    call_tauri_simple("get_all_batches").await
}

/// Pause a batch
pub async fn pause_batch(batch_id: String) -> Result<TauriResult<bool>, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PauseBatchArgs {
        batch_id: String,
    }

    let args = PauseBatchArgs { batch_id };
    call_tauri("pause_batch", args).await
}

/// Resume a paused batch
pub async fn resume_batch(batch_id: String) -> Result<TauriResult<bool>, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ResumeBatchArgs {
        batch_id: String,
    }

    let args = ResumeBatchArgs { batch_id };
    call_tauri("resume_batch", args).await
}

/// Cancel a batch
pub async fn cancel_batch(batch_id: String) -> Result<TauriResult<bool>, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct CancelBatchArgs {
        batch_id: String,
    }

    let args = CancelBatchArgs { batch_id };
    call_tauri("cancel_batch", args).await
}

/// Remove a completed batch
pub async fn remove_batch(batch_id: String) -> Result<TauriResult<bool>, TauriError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct RemoveBatchArgs {
        batch_id: String,
    }

    let args = RemoveBatchArgs { batch_id };
    call_tauri("remove_batch", args).await
}

/// Clean up finished batches
pub async fn cleanup_finished_batches() -> Result<TauriResult<bool>, TauriError> {
    call_tauri_simple("cleanup_finished_batches").await
}

/// Open directory dialog for selecting download folder
pub async fn open_directory_dialog() -> Result<TauriResult<Option<String>>, TauriError> {
    call_tauri_simple("open_directory_dialog").await
}

// ============================================================================
// COMPRESSION TYPES
// ============================================================================

/// Compression result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressionResult {
    pub id: String,
    pub input_path: String,
    pub output_path: String,
    pub original_size_mb: u64,
    pub compressed_size_mb: u64,
    pub compression_ratio: f64,
    pub quality_used: CompressionQuality,
    pub processing_time_seconds: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Compression progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionProgress {
    pub id: String,
    pub status: CompressionStatus,
    pub progress_percentage: f64,
    pub current_frame: Option<u64>,
    pub total_frames: Option<u64>,
    pub fps: Option<f64>,
    pub speed: Option<String>,
    pub eta_seconds: Option<u64>,
    pub input_size_mb: u64,
    pub estimated_output_size_mb: u64,
    pub actual_output_size_mb: Option<u64>,
    pub compression_ratio: Option<f64>,
    pub error_message: Option<String>,
}

/// Compression status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionStatus {
    Queued,
    Analyzing,
    Compressing,
    Completed,
    Failed,
    Cancelled,
}

// ============================================================================
// BATCH PROCESSING TYPES
// ============================================================================

/// Status of a batch processing job
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchStatus {
    Queued,
    Processing,
    Completed,
    Paused,
    Cancelled,
    Failed,
}

/// Status of an individual item within a batch
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BatchItemStatus {
    Queued,
    Validating,
    Extracting,
    Downloading,
    Compressing,
    Completed,
    Failed,
    Skipped,
    Cancelled,
}

/// Individual item in a batch processing job
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchItem {
    pub id: String,
    pub url: String,
    pub status: BatchItemStatus,
    pub video_info: Option<VideoInfo>,
    pub quality_index: Option<usize>,
    pub download_progress: Option<DownloadProgress>,
    pub compression_result: Option<CompressionResult>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub output_path: Option<String>,
}

/// Options for batch processing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchOptions {
    pub max_concurrent_downloads: usize,
    pub max_concurrent_extractions: usize,
    pub default_quality_index: Option<usize>,
    pub enable_compression: bool,
    pub compression_quality: CompressionQuality,
    pub preserve_original_on_compression: bool,
    pub output_directory: Option<String>,
    pub retry_failed_items: bool,
    pub max_retries_per_item: usize,
    pub continue_on_errors: bool,
}

impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: 3,
            max_concurrent_extractions: 5,
            default_quality_index: None,
            enable_compression: false,
            compression_quality: CompressionQuality::Medium,
            preserve_original_on_compression: false,
            output_directory: None,
            retry_failed_items: true,
            max_retries_per_item: 2,
            continue_on_errors: true,
        }
    }
}

/// Progress information for a batch processing job
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchProgress {
    pub id: String,
    pub status: BatchStatus,
    pub total_items: usize,
    pub completed_items: usize,
    pub failed_items: usize,
    pub active_items: usize,
    pub queued_items: usize,
    pub progress_percentage: f64,
    pub eta_seconds: Option<u64>,
    pub items_per_minute: Option<f64>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub total_downloaded_bytes: u64,
    pub current_download_speed_bps: Option<u64>,
}

/// A batch processing job containing multiple video URLs to process
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchJob {
    pub id: String,
    pub name: String,
    pub status: BatchStatus,
    pub items: Vec<BatchItem>,
    pub options: BatchOptions,
    pub progress: BatchProgress,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Statistics for a completed or ongoing batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatistics {
    pub total_processing_time_seconds: u64,
    pub average_time_per_item_seconds: f64,
    pub success_rate_percentage: f64,
    pub total_downloaded_mb: u64,
    pub total_compression_savings_mb: u64,
    pub most_common_failure_reason: Option<String>,
}
