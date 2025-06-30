use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use crate::services::tauri_api::{self, BatchJob, BatchProgress, BatchStatus, BatchOptions};
use crate::utils::validation::is_valid_facebook_url;
use crate::components::{LoadingIcon, compression::CompressionQuality, BatchQueueView, ItemAction, BulkAction};
use crate::hooks::use_local_storage::use_local_storage;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum BatchPageState {
    Input,
    Processing,
    Completed,
    Error(String),
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchPagePersistentState {
    pub urls_text: String,
    pub batch_name: String,
    pub page_state: BatchPageState,
    pub current_batch: Option<BatchJob>,
    pub batch_progress: Option<BatchProgress>,
    pub enable_compression: bool,
    pub compression_quality: CompressionQuality,
    pub max_concurrent: usize,
    pub output_directory: String,
}

impl Default for BatchPagePersistentState {
    fn default() -> Self {
        Self {
            urls_text: String::new(),
            batch_name: String::new(),
            page_state: BatchPageState::Input,
            current_batch: None,
            batch_progress: None,
            enable_compression: false,
            compression_quality: CompressionQuality::Medium,
            max_concurrent: 3,
            output_directory: String::new(),
        }
    }
}

#[function_component(BatchPage)]
pub fn batch_page() -> Html {
    // Use persistent state for all batch page data
    let (persistent_state, set_persistent_state) = use_local_storage(
        "facebook_downloader_batch_state",
        BatchPagePersistentState::default()
    );



    // Progress polling effect
    {
        let current_batch_id = persistent_state.current_batch.as_ref().map(|b| b.id.clone());
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state_clone = persistent_state.clone();

        use_effect_with(current_batch_id.clone(), move |batch_id_opt| {
            if let Some(batch_id) = batch_id_opt.as_ref() {
                let batch_id = batch_id.clone();
                let set_persistent_state = set_persistent_state.clone();
                let persistent_state = persistent_state_clone.clone();

                console::log_1(&format!("🔄 Starting polling for batch: {} (current page state: {:?})", batch_id, persistent_state.page_state).into());

                spawn_local(async move {
                    let mut poll_count = 0;
                    loop {
                        poll_count += 1;
                        console::log_1(&format!("🔄 Poll #{} for batch: {}", poll_count, batch_id).into());
                        // Fetch full batch info to get updated item statuses
                        match tauri_api::get_batch_info(batch_id.clone()).await {
                            Ok(result) => {
                                if let Some(batch) = result.data {
                                    let is_finished = matches!(
                                        batch.status,
                                        BatchStatus::Completed | BatchStatus::Failed | BatchStatus::Cancelled
                                    );

                                    // Debug logging for UI updates
                                    console::log_1(&format!("📊 UI Update: {}% ({}/{} completed, {} active, {} failed)",
                                        batch.progress.progress_percentage,
                                        batch.progress.completed_items,
                                        batch.progress.total_items,
                                        batch.progress.active_items,
                                        batch.progress.failed_items).into());

                                    // Update persistent state with new batch info and progress
                                    let mut new_state = persistent_state.clone();
                                    new_state.current_batch = Some(batch.clone());
                                    new_state.batch_progress = Some(batch.progress.clone());

                                    // CRITICAL FIX: Only change page state when transitioning to completed
                                    // Don't force back to Processing if user is already in a different state
                                    if is_finished {
                                        new_state.page_state = BatchPageState::Completed;
                                        console::log_1(&"🏁 Setting page state to Completed".into());
                                    } else if matches!(persistent_state.page_state, BatchPageState::Input) {
                                        // Only set to Processing if we're coming from Input state
                                        new_state.page_state = BatchPageState::Processing;
                                        console::log_1(&"🔄 Setting page state to Processing from Input".into());
                                    } else {
                                        // Preserve current page state (don't force back to Processing)
                                        console::log_1(&format!("🔄 Preserving current page state: {:?}", persistent_state.page_state).into());
                                    }

                                    set_persistent_state.emit(new_state);

                                    if is_finished {
                                        console::log_1(&format!("✅ Batch finished, stopping polling: {:?}", batch.status).into());
                                        break;
                                    }
                                } else {
                                    // Batch not found, might have been removed
                                    console::log_1(&"❌ Batch not found, stopping polling".into());
                                    break;
                                }
                            }
                            Err(e) => {
                                console::log_1(&format!("❌ Failed to get batch info: {}", e.error).into());
                                // Try to fall back to progress-only update
                                match tauri_api::get_batch_progress(batch_id.clone()).await {
                                    Ok(progress_result) => {
                                        if let Some(progress) = progress_result.data {
                                            let mut new_state = persistent_state.clone();
                                            new_state.batch_progress = Some(progress.clone());

                                            let is_finished = matches!(
                                                progress.status,
                                                BatchStatus::Completed | BatchStatus::Failed | BatchStatus::Cancelled
                                            );

                                            // CRITICAL FIX: Only change page state when transitioning to completed
                                            // Don't force back to Processing if user is already in a different state
                                            if is_finished {
                                                new_state.page_state = BatchPageState::Completed;
                                            } else if matches!(persistent_state.page_state, BatchPageState::Input) {
                                                // Only set to Processing if we're coming from Input state
                                                new_state.page_state = BatchPageState::Processing;
                                            }
                                            // Otherwise preserve current page state

                                            set_persistent_state.emit(new_state);

                                            if is_finished {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                    Err(_) => {
                                        console::log_1(&"Failed to get batch progress as fallback, stopping polling".into());
                                        break;
                                    }
                                }
                            }
                        }

                        // TIMEOUT FIX: More patient polling for Facebook extractions
                        // Facebook reels can take 38-58 seconds, so we need longer intervals
                        let poll_interval = if poll_count <= 5 {
                            250  // Fast polling for first 5 polls (1.25 seconds)
                        } else if poll_count <= 20 {
                            1000 // Medium polling for next 15 polls (15 seconds total)
                        } else {
                            2000 // Slower polling after that (Facebook reels need time)
                        };
                        gloo::timers::future::TimeoutFuture::new(poll_interval).await;
                    }
                });
            }
            || {}
        });
    }

    let on_urls_input = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            let new_value = input.value();
            console::log_1(&format!("🔄 URLs input changed: {}", new_value).into());
            let mut new_state = persistent_state.clone();
            new_state.urls_text = new_value;
            set_persistent_state.emit(new_state);
        })
    };

    let on_urls_change = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            let new_value = input.value();
            console::log_1(&format!("� URLs changed (onchange): {}", new_value).into());
            let mut new_state = persistent_state.clone();
            new_state.urls_text = new_value;
            set_persistent_state.emit(new_state);
        })
    };

    let on_batch_name_change = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut new_state = persistent_state.clone();
            new_state.batch_name = input.value();
            set_persistent_state.emit(new_state);
        })
    };

    let on_output_directory_change = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut new_state = persistent_state.clone();
            new_state.output_directory = input.value();
            set_persistent_state.emit(new_state);
        })
    };

    let on_browse_directory = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();
        Callback::from(move |_| {
            let set_persistent_state = set_persistent_state.clone();
            let persistent_state = persistent_state.clone();

            spawn_local(async move {
                match tauri_api::open_directory_dialog().await {
                    Ok(result) => {
                        if let Some(path) = result.data {
                            let mut new_state = persistent_state.clone();
                            new_state.output_directory = path;
                            set_persistent_state.emit(new_state);
                        }
                    }
                    Err(e) => {
                        console::log_1(&format!("Failed to open directory dialog: {}", e.error).into());
                    }
                }
            });
        })
    };

    let on_compression_toggle = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();
        Callback::from(move |_| {
            let mut new_state = persistent_state.clone();
            new_state.enable_compression = !new_state.enable_compression;
            set_persistent_state.emit(new_state);
        })
    };

    let on_compression_quality_change = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let quality = match select.value().as_str() {
                "High" => CompressionQuality::High,
                "Medium" => CompressionQuality::Medium,
                "Low" => CompressionQuality::Low,
                "Minimal" => CompressionQuality::Minimal,
                _ => CompressionQuality::Medium,
            };
            let mut new_state = persistent_state.clone();
            new_state.compression_quality = quality;
            set_persistent_state.emit(new_state);
        })
    };

    let on_max_concurrent_change = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<usize>() {
                let mut new_state = persistent_state.clone();
                new_state.max_concurrent = value.max(1).min(10);
                set_persistent_state.emit(new_state);
            }
        })
    };

    let on_start_batch = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();

        Callback::from(move |_| {
            let urls = persistent_state.urls_text.clone();
            let name = persistent_state.batch_name.clone();

            if urls.trim().is_empty() {
                return;
            }

            let url_list: Vec<String> = urls
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();

            if url_list.is_empty() {
                return;
            }

            let batch_name = if name.trim().is_empty() {
                let date = js_sys::Date::new_0();
                format!("Batch {}", date.to_iso_string().as_string().unwrap_or_else(|| "Unknown".to_string())[..16].replace('T', " "))
            } else {
                name
            };

            let options = BatchOptions {
                max_concurrent_downloads: persistent_state.max_concurrent,
                // PERFORMANCE FIX: Increase extraction concurrency for better throughput
                max_concurrent_extractions: (persistent_state.max_concurrent * 3).min(15),
                default_quality_index: Some(0), // Best quality
                enable_compression: persistent_state.enable_compression,
                compression_quality: persistent_state.compression_quality,
                preserve_original_on_compression: false,
                output_directory: if persistent_state.output_directory.trim().is_empty() {
                    // Use platform-specific default downloads directory
                    #[cfg(target_os = "windows")]
                    let default_dir = format!("{}\\Downloads\\FacebookVideos", std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string()));

                    #[cfg(not(target_os = "windows"))]
                    let default_dir = format!("{}/Downloads/FacebookVideos", std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()));

                    Some(default_dir)
                } else {
                    Some(persistent_state.output_directory.clone())
                },
                retry_failed_items: true,
                // TIMEOUT FIX: Increase retries for slow Facebook extractions
                max_retries_per_item: 3,
                continue_on_errors: true,
            };

            // CRITICAL FIX: Set Processing state IMMEDIATELY (synchronously) before async work
            console::log_1(&"🚀 Setting initial page state to Processing (SYNCHRONOUS)".into());
            let mut initial_state = persistent_state.clone();
            initial_state.page_state = BatchPageState::Processing;

            // Create initial progress state to show immediate feedback
            let initial_progress = BatchProgress {
                id: "pending".to_string(),
                status: BatchStatus::Processing,
                total_items: url_list.len(),
                completed_items: 0,
                failed_items: 0,
                active_items: 0,
                queued_items: url_list.len(),
                progress_percentage: 0.0,
                eta_seconds: None,
                items_per_minute: None,
                started_at: None,
                completed_at: None,
                total_downloaded_bytes: 0,
                current_download_speed_bps: None,
            };
            initial_state.batch_progress = Some(initial_progress);
            console::log_1(&"📤 Emitting initial Processing state (SYNCHRONOUS)".into());
            set_persistent_state.emit(initial_state.clone());

            let set_persistent_state = set_persistent_state.clone();
            let persistent_state = persistent_state.clone();

            spawn_local(async move {

                console::log_1(&"🚀 Starting batch processing...".into());

                match tauri_api::start_batch_processing(batch_name, url_list, Some(options)).await {
                    Ok(result) => {
                        let batch_id = result.data;
                        console::log_1(&format!("✅ Batch started with ID: {}", batch_id).into());

                        // Immediately try to get initial batch info with retries
                        let mut retry_count = 0;
                        const MAX_RETRIES: u32 = 10;

                        loop {
                            match tauri_api::get_batch_info(batch_id.clone()).await {
                                Ok(batch_result) => {
                                    if let Some(batch) = batch_result.data {
                                        console::log_1(&format!("📊 Got initial batch info: {}% progress", batch.progress.progress_percentage).into());
                                        let mut new_state = persistent_state.clone();
                                        new_state.page_state = BatchPageState::Processing; // Ensure Processing state is maintained
                                        new_state.current_batch = Some(batch.clone());
                                        new_state.batch_progress = Some(batch.progress.clone());
                                        set_persistent_state.emit(new_state);
                                        break;
                                    } else if retry_count < MAX_RETRIES {
                                        retry_count += 1;
                                        console::log_1(&format!("⏳ Batch not ready yet, retrying... ({}/{})", retry_count, MAX_RETRIES).into());
                                        gloo::timers::future::TimeoutFuture::new(100).await;
                                        continue;
                                    } else {
                                        console::log_1(&"❌ Failed to get initial batch info after retries".into());
                                        break;
                                    }
                                }
                                Err(e) => {
                                    console::log_1(&format!("❌ Failed to get batch info: {}", e.error).into());
                                    if retry_count < MAX_RETRIES {
                                        retry_count += 1;
                                        gloo::timers::future::TimeoutFuture::new(100).await;
                                        continue;
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        console::log_1(&format!("❌ Failed to start batch: {}", e.error).into());
                        let mut new_state = persistent_state.clone();
                        new_state.page_state = BatchPageState::Error(e.error);
                        set_persistent_state.emit(new_state);
                    }
                }
            });
        })
    };

    let on_clear = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();
        Callback::from(move |_| {
            console::log_1(&"🧹 Clearing batch input".into());
            let mut new_state = persistent_state.clone();
            new_state.urls_text = String::new();
            new_state.batch_name = String::new();
            set_persistent_state.emit(new_state);
        })
    };

    let on_reset_all = {
        let set_persistent_state = set_persistent_state.clone();
        Callback::from(move |_| {
            console::log_1(&"🔄 Resetting all batch state".into());
            let new_state = BatchPagePersistentState::default();
            set_persistent_state.emit(new_state);
        })
    };

    let on_new_batch = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();

        Callback::from(move |_| {
            let mut new_state = persistent_state.clone();
            new_state.page_state = BatchPageState::Input;
            new_state.current_batch = None;
            new_state.batch_progress = None;
            new_state.urls_text = String::new();
            new_state.batch_name = String::new();
            set_persistent_state.emit(new_state);
        })
    };

    let on_resume_batch = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();

        Callback::from(move |_| {
            if let Some(batch) = &persistent_state.current_batch {
                let batch_id = batch.id.clone();
                let set_persistent_state = set_persistent_state.clone();
                let persistent_state = persistent_state.clone();

                console::log_1(&format!("🔄 Resuming batch: {}", batch_id).into());

                spawn_local(async move {
                    match tauri_api::resume_batch(batch_id.clone()).await {
                        Ok(_) => {
                            console::log_1(&"✅ Batch resumed successfully".into());
                            // Switch back to processing state
                            let mut new_state = persistent_state.clone();
                            new_state.page_state = BatchPageState::Processing;
                            set_persistent_state.emit(new_state);
                        }
                        Err(e) => {
                            console::log_1(&format!("❌ Failed to resume batch: {}", e.error).into());
                            let mut new_state = persistent_state.clone();
                            new_state.page_state = BatchPageState::Error(
                                format!("Failed to resume batch: {}", e.error)
                            );
                            set_persistent_state.emit(new_state);
                        }
                    }
                });
            }
        })
    };

    let on_delete_batch = {
        let set_persistent_state = set_persistent_state.clone();
        let persistent_state = persistent_state.clone();

        Callback::from(move |_| {
            if let Some(batch) = &persistent_state.current_batch {
                let batch_id = batch.id.clone();
                let set_persistent_state = set_persistent_state.clone();

                console::log_1(&format!("🗑️ Deleting batch: {}", batch_id).into());

                spawn_local(async move {
                    match tauri_api::remove_batch(batch_id.clone()).await {
                        Ok(_) => {
                            console::log_1(&"✅ Batch deleted successfully".into());
                            // Clear state and go back to input
                            let new_state = BatchPagePersistentState::default();
                            set_persistent_state.emit(new_state);
                        }
                        Err(e) => {
                            console::log_1(&format!("❌ Failed to delete batch: {}", e.error).into());
                            // Even if delete fails, clear the local state
                            let new_state = BatchPagePersistentState::default();
                            set_persistent_state.emit(new_state);
                        }
                    }
                });
            }
        })
    };

    let url_count = persistent_state.urls_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    let valid_url_count = persistent_state.urls_text
        .lines()
        .filter(|line| !line.trim().is_empty() && is_valid_facebook_url(line.trim()))
        .count();

    // Debug log for state rendering
    console::log_1(&format!("🎨 Rendering page state: {:?}", persistent_state.page_state).into());

    html! {
        <div class="p-6">
            <div class="max-w-6xl mx-auto">
                <div class="mb-6">
                    <h1 class="text-2xl font-bold text-gray-800 dark:text-white mb-2">{"Batch Download"}</h1>
                    <p class="text-gray-600 dark:text-gray-300">
                        {"Download multiple Facebook videos simultaneously with advanced options and progress tracking."}
                    </p>
                </div>

                {match &persistent_state.page_state {
                    BatchPageState::Input => render_input_form(
                        &persistent_state,
                        url_count,
                        valid_url_count,
                        on_urls_input,
                        on_urls_change,
                        on_batch_name_change,
                        on_output_directory_change,
                        on_browse_directory,
                        on_compression_toggle,
                        on_compression_quality_change,
                        on_max_concurrent_change,
                        on_start_batch,
                        on_clear,
                        on_reset_all,
                    ),
                    BatchPageState::Processing => render_processing_view(
                        persistent_state.current_batch.as_ref(),
                        persistent_state.batch_progress.as_ref(),
                        on_new_batch.clone(),
                    ),
                    BatchPageState::Completed => render_completed_view(
                        persistent_state.current_batch.as_ref(),
                        persistent_state.batch_progress.as_ref(),
                        on_new_batch,
                    ),
                    BatchPageState::Error(error) => render_error_view(
                        error,
                        on_new_batch,
                        persistent_state.current_batch.as_ref(),
                        Some(on_resume_batch),
                        Some(on_delete_batch),
                    ),
                }}
            </div>
        </div>
    }
}

fn render_input_form(
    persistent_state: &BatchPagePersistentState,
    url_count: usize,
    valid_url_count: usize,
    on_urls_input: Callback<InputEvent>,
    on_urls_change: Callback<Event>,
    on_batch_name_change: Callback<Event>,
    on_output_directory_change: Callback<Event>,
    on_browse_directory: Callback<MouseEvent>,
    on_compression_toggle: Callback<MouseEvent>,
    on_compression_quality_change: Callback<Event>,
    on_max_concurrent_change: Callback<Event>,
    on_start_batch: Callback<MouseEvent>,
    on_clear: Callback<MouseEvent>,
    on_reset_all: Callback<MouseEvent>,
) -> Html {
    html! {
        <>
            // Batch Configuration
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700 mb-6">
                <h3 class="text-lg font-semibold text-gray-800 dark:text-white mb-4">{"Batch Configuration"}</h3>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            {"Batch Name"}
                        </label>
                        <input
                            type="text"
                            class="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-facebook-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400"
                            placeholder="Enter batch name (optional)"
                            value={persistent_state.batch_name.clone()}
                            onchange={on_batch_name_change}
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            {"Output Directory"}
                        </label>
                        <div class="flex gap-2">
                            <input
                                type="text"
                                class="flex-1 p-3 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-facebook-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400"
                                placeholder="Leave empty for default Downloads/FacebookVideos"
                                value={persistent_state.output_directory.clone()}
                                onchange={on_output_directory_change}
                            />
                            <button
                                type="button"
                                class="px-4 py-3 bg-gray-100 hover:bg-gray-200 dark:bg-gray-600 dark:hover:bg-gray-500 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-700 dark:text-gray-300 transition-colors duration-200 flex items-center gap-2"
                                onclick={on_browse_directory}
                                title="Browse for folder"
                            >
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-5l-2-2H6a2 2 0 00-2 2z"></path>
                                </svg>
                                {"Browse"}
                            </button>
                        </div>
                    </div>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            {"Max Concurrent Downloads"}
                            <span class="text-xs text-gray-500 dark:text-gray-400 block">
                                {"Recommended: 1-3 for Facebook reels"}
                            </span>
                        </label>
                        <input
                            type="number"
                            min="1"
                            max="10"
                            class="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-facebook-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                            value={persistent_state.max_concurrent.to_string()}
                            onchange={on_max_concurrent_change}
                        />
                        <div class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                            {if persistent_state.max_concurrent > 5 {
                                "⚠️ High concurrency may cause timeouts with Facebook"
                            } else if persistent_state.max_concurrent <= 2 {
                                "✅ Conservative setting - good for stability"
                            } else {
                                "✅ Balanced setting - good for most cases"
                            }}
                        </div>
                    </div>

                    <div class="flex items-center">
                        <label class="flex items-center cursor-pointer">
                            <input
                                type="checkbox"
                                class="sr-only"
                                checked={persistent_state.enable_compression}
                                onclick={on_compression_toggle}
                            />
                            <div class={format!(
                                "relative w-11 h-6 rounded-full transition-colors duration-200 {}",
                                if persistent_state.enable_compression {
                                    "bg-facebook-500"
                                } else {
                                    "bg-gray-300 dark:bg-gray-600"
                                }
                            )}>
                                <div class={format!(
                                    "absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform duration-200 {}",
                                    if persistent_state.enable_compression { "translate-x-5" } else { "translate-x-0" }
                                )}></div>
                            </div>
                            <span class="ml-3 text-sm font-medium text-gray-700 dark:text-gray-300">
                                {"Enable Compression"}
                            </span>
                        </label>
                    </div>

                    {if persistent_state.enable_compression {
                        html! {
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    {"Compression Quality"}
                                </label>
                                <select
                                    class="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-facebook-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                                    onchange={on_compression_quality_change}
                                >
                                    <option value="High" selected={matches!(persistent_state.compression_quality, CompressionQuality::High)}>{"High (90%)"}</option>
                                    <option value="Medium" selected={matches!(persistent_state.compression_quality, CompressionQuality::Medium)}>{"Medium (50%)"}</option>
                                    <option value="Low" selected={matches!(persistent_state.compression_quality, CompressionQuality::Low)}>{"Low (30%)"}</option>
                                    <option value="Minimal" selected={matches!(persistent_state.compression_quality, CompressionQuality::Minimal)}>{"Minimal (10%)"}</option>
                                </select>
                            </div>
                        }
                    } else {
                        html! { <div></div> }
                    }}
                </div>
            </div>

            // URL Input
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700 mb-6">
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        {"Facebook Video URLs"}
                    </label>
                    <textarea
                        class="w-full h-64 p-3 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-facebook-500 focus:border-transparent resize-none bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 transition-colors duration-200"
                        placeholder="https://www.facebook.com/watch?v=...
https://www.facebook.com/username/videos/...
https://fb.watch/..."
                        value={persistent_state.urls_text.clone()}
                        oninput={on_urls_input}
                        onchange={on_urls_change}
                    />
                    <div class="flex justify-between items-center mt-2">
                        <div class="flex space-x-4">
                            <span class="text-sm text-gray-500 dark:text-gray-400">
                                {format!("{} URLs detected", url_count)}
                            </span>
                            <span class={format!(
                                "text-sm {}",
                                if valid_url_count == url_count && url_count > 0 {
                                    "text-green-600 dark:text-green-400"
                                } else if valid_url_count > 0 {
                                    "text-yellow-600 dark:text-yellow-400"
                                } else {
                                    "text-red-600 dark:text-red-400"
                                }
                            )}>
                                {format!("{} valid URLs", valid_url_count)}
                            </span>
                        </div>
                        <div class="flex space-x-3">
                            <button
                                class="text-sm text-facebook-600 hover:text-facebook-800 dark:text-facebook-400 dark:hover:text-facebook-300 transition-colors duration-200"
                                onclick={on_clear}
                            >
                                {"Clear URLs"}
                            </button>
                            <button
                                class="text-sm text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300 transition-colors duration-200"
                                onclick={on_reset_all}
                                title="Reset all settings and clear everything"
                            >
                                {"Reset All"}
                            </button>
                        </div>
                    </div>
                </div>

                // Action Buttons
                <div class="flex justify-between items-center">
                    <div class="text-sm text-gray-600 dark:text-gray-400">
                        {if url_count > 0 {
                            format!("Ready to process {} video{}", url_count, if url_count == 1 { "" } else { "s" })
                        } else {
                            "Enter URLs above to get started".to_string()
                        }}
                    </div>
                    <button
                        class={format!(
                            "px-6 py-3 text-white rounded-lg font-medium transition-colors duration-200 {}",
                            if valid_url_count > 0 {
                                "bg-facebook-500 hover:bg-facebook-600 dark:bg-facebook-600 dark:hover:bg-facebook-700"
                            } else {
                                "bg-gray-400 cursor-not-allowed"
                            }
                        )}
                        onclick={on_start_batch}
                        disabled={valid_url_count == 0}
                    >
                        {format!("Start Batch Download ({})", valid_url_count)}
                    </button>
                </div>
            </div>

            // Tips Section
            <div class="bg-facebook-50 dark:bg-facebook-900/20 border border-facebook-200 dark:border-facebook-800 rounded-lg p-4">
                <h3 class="text-sm font-semibold text-facebook-800 dark:text-facebook-300 mb-2">{"Batch Processing Tips"}</h3>
                <ul class="text-sm text-facebook-700 dark:text-facebook-400 space-y-1">
                    <li>{"• Ensure all URLs are valid Facebook video links"}</li>
                    <li>{"• Large batches are processed concurrently for faster completion"}</li>
                    <li>{"• Failed downloads will be automatically retried up to 3 times"}</li>
                    <li>{"• Facebook reels may take 30-60 seconds to extract - this is normal"}</li>
                    <li>{"• Share links extract faster (4-5 seconds) than reel links"}</li>
                    <li>{"• You can monitor individual video progress in real-time"}</li>
                    <li>{"• Compression reduces file size while maintaining quality"}</li>
                    <li>{"• Small file size estimates are common but don't affect actual downloads"}</li>
                </ul>
            </div>
        </>
    }
}

fn render_processing_view(
    current_batch: Option<&BatchJob>,
    batch_progress: Option<&BatchProgress>,
    on_reset_batch: Callback<MouseEvent>,
) -> Html {
    html! {
        <div class="space-y-6">
            // Overall Progress
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
                <h3 class="text-lg font-semibold text-gray-800 dark:text-white mb-4">{"Batch Processing"}</h3>

                {if let Some(progress) = batch_progress {
                    html! {
                        <>
                            <div class="mb-4">
                                <div class="flex justify-between items-center mb-2">
                                    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                                        {"Overall Progress"}
                                    </span>
                                    <div class="flex items-center space-x-2">
                                        <span class="text-sm text-gray-500 dark:text-gray-400">
                                            {format!("{:.1}%", progress.progress_percentage)}
                                        </span>
                                        {if progress.active_items > 0 {
                                            html! {
                                                <div class="flex items-center space-x-1">
                                                    <div class="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
                                                    <span class="text-xs text-blue-600 dark:text-blue-400">
                                                        {"Processing..."}
                                                    </span>
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        }}
                                    </div>
                                </div>
                                <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                                    <div
                                        class="bg-facebook-500 h-2 rounded-full transition-all duration-300"
                                        style={format!("width: {}%", progress.progress_percentage)}
                                    ></div>
                                </div>
                                {if progress.active_items > 0 && progress.progress_percentage < 100.0 {
                                    html! {
                                        <div class="mt-2 text-xs text-gray-500 dark:text-gray-400 text-center">
                                            {format!("Extracting {} video{} from Facebook - this may take 30-60 seconds per reel",
                                                progress.active_items,
                                                if progress.active_items == 1 { "" } else { "s" }
                                            )}
                                        </div>
                                    }
                                } else if progress.progress_percentage == 100.0 {
                                    html! {
                                        <div class="mt-2 text-xs text-green-600 dark:text-green-400 text-center">
                                            {"✅ All extractions completed successfully!"}
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>

                            <div class="grid grid-cols-2 md:grid-cols-5 gap-4 text-center">
                                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-3">
                                    <div class="text-2xl font-bold text-gray-800 dark:text-white">
                                        {progress.total_items}
                                    </div>
                                    <div class="text-xs text-gray-500 dark:text-gray-400">{"Total"}</div>
                                </div>
                                <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-3">
                                    <div class="text-2xl font-bold text-blue-600 dark:text-blue-400">
                                        {progress.active_items}
                                    </div>
                                    <div class="text-xs text-blue-500 dark:text-blue-400">{"Active"}</div>
                                </div>
                                <div class="bg-yellow-50 dark:bg-yellow-900/20 rounded-lg p-3">
                                    <div class="text-2xl font-bold text-yellow-600 dark:text-yellow-400">
                                        {progress.queued_items}
                                    </div>
                                    <div class="text-xs text-yellow-500 dark:text-yellow-400">{"Queued"}</div>
                                </div>
                                <div class="bg-green-50 dark:bg-green-900/20 rounded-lg p-3">
                                    <div class="text-2xl font-bold text-green-600 dark:text-green-400">
                                        {progress.completed_items}
                                    </div>
                                    <div class="text-xs text-green-500 dark:text-green-400">{"Completed"}</div>
                                </div>
                                <div class="bg-red-50 dark:bg-red-900/20 rounded-lg p-3">
                                    <div class="text-2xl font-bold text-red-600 dark:text-red-400">
                                        {progress.failed_items}
                                    </div>
                                    <div class="text-xs text-red-500 dark:text-red-400">{"Failed"}</div>
                                </div>
                            </div>

                            <div class="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4 text-center">
                                {if let Some(speed) = progress.current_download_speed_bps {
                                    html! {
                                        <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-3">
                                            <div class="text-sm text-gray-600 dark:text-gray-400">
                                                {"Download Speed"}
                                            </div>
                                            <div class="text-lg font-semibold text-blue-600 dark:text-blue-400">
                                                {format!("{:.1} MB/s", speed as f64 / 1024.0 / 1024.0)}
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}

                                {if progress.total_downloaded_bytes > 0 {
                                    html! {
                                        <div class="bg-green-50 dark:bg-green-900/20 rounded-lg p-3">
                                            <div class="text-sm text-gray-600 dark:text-gray-400">
                                                {"Total Downloaded"}
                                            </div>
                                            <div class="text-lg font-semibold text-green-600 dark:text-green-400">
                                                {format!("{:.1} MB", progress.total_downloaded_bytes as f64 / 1024.0 / 1024.0)}
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                        </>
                    }
                } else {
                    html! {
                        <div class="flex items-center justify-center py-8">
                            <LoadingIcon />
                            <span class="ml-3 text-gray-600 dark:text-gray-400">{"Loading batch information..."}</span>
                        </div>
                    }
                }}
            </div>

            // Facebook Extraction Info and Controls
            <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
                <div class="flex justify-between items-start mb-2">
                    <h3 class="text-sm font-semibold text-blue-800 dark:text-blue-300 flex items-center">
                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                        </svg>
                        {"Facebook Extraction Information"}
                    </h3>
                    <button
                        class="px-3 py-1 bg-red-500 hover:bg-red-600 dark:bg-red-600 dark:hover:bg-red-700 text-white text-xs rounded font-medium transition-colors duration-200"
                        onclick={on_reset_batch}
                        title="Reset batch if it appears stuck or unresponsive"
                    >
                        {"🔄 Reset Batch"}
                    </button>
                </div>
                <div class="text-sm text-blue-700 dark:text-blue-400 space-y-1">
                    <p>{"• Facebook share links (share/r/...) extract quickly (4-5 seconds)"}</p>
                    <p>{"• Facebook reel links (reel/...) take longer (30-60 seconds) - this is normal"}</p>
                    <p>{"• Videos showing as \"Processing\" are being extracted from Facebook's servers"}</p>
                    <p>{"• File size estimates may appear small but actual downloads will be correct"}</p>
                    <p>{"• Failed extractions will be automatically retried up to 3 times"}</p>
                    <p class="text-yellow-700 dark:text-yellow-400 font-medium">{"• If the batch appears stuck, click \"Reset Batch\" to clear the state"}</p>
                </div>
            </div>

            // Individual Items Progress
            {if let Some(batch) = current_batch {
                html! {
                    <BatchQueueView
                        items={batch.items.clone()}
                        on_item_action={Some(Callback::from(|(_item_id, _action): (String, ItemAction)| {
                            // TODO: Implement item actions (retry, pause, cancel, etc.)
                            console::log_1(&"Item action triggered".into());
                        }))}
                        on_bulk_action={Some(Callback::from(|_action: BulkAction| {
                            // TODO: Implement bulk actions (retry all failed, etc.)
                            console::log_1(&"Bulk action triggered".into());
                        }))}
                    />
                }
            } else {
                html! {}
            }}
        </div>
    }
}



fn render_completed_view(
    current_batch: Option<&BatchJob>,
    batch_progress: Option<&BatchProgress>,
    on_new_batch: Callback<MouseEvent>,
) -> Html {
    html! {
        <div class="space-y-6">
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
                <div class="text-center">
                    <div class="w-16 h-16 mx-auto mb-4 bg-green-100 dark:bg-green-900/20 rounded-full flex items-center justify-center">
                        <svg class="w-8 h-8 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                        </svg>
                    </div>
                    <h3 class="text-xl font-semibold text-gray-800 dark:text-white mb-2">{"Batch Completed!"}</h3>

                    {if let Some(progress) = batch_progress {
                        html! {
                            <div class="text-gray-600 dark:text-gray-400 mb-6">
                                {format!(
                                    "Successfully processed {} out of {} videos",
                                    progress.completed_items,
                                    progress.total_items
                                )}
                                {if progress.failed_items > 0 {
                                    format!(" ({} failed)", progress.failed_items)
                                } else {
                                    String::new()
                                }}
                            </div>
                        }
                    } else {
                        html! {}
                    }}

                    <button
                        class="px-6 py-3 bg-facebook-500 hover:bg-facebook-600 dark:bg-facebook-600 dark:hover:bg-facebook-700 text-white rounded-lg font-medium transition-colors duration-200"
                        onclick={on_new_batch}
                    >
                        {"Start New Batch"}
                    </button>
                </div>
            </div>

            // Show final results
            {if let Some(batch) = current_batch {
                html! {
                    <BatchQueueView
                        items={batch.items.clone()}
                        on_item_action={Some(Callback::from(|(_item_id, _action): (String, ItemAction)| {
                            // TODO: Implement item actions for completed view
                            console::log_1(&"Completed view item action triggered".into());
                        }))}
                        on_bulk_action={Some(Callback::from(|_action: BulkAction| {
                            // TODO: Implement bulk actions for completed view
                            console::log_1(&"Completed view bulk action triggered".into());
                        }))}
                    />
                }
            } else {
                html! {}
            }}
        </div>
    }
}

fn render_error_view(
    error: &str,
    on_new_batch: Callback<MouseEvent>,
    current_batch: Option<&BatchJob>,
    on_resume_batch: Option<Callback<MouseEvent>>,
    on_delete_batch: Option<Callback<MouseEvent>>,
) -> Html {
    // Check if this is a timeout-related error
    let is_timeout_error = error.to_lowercase().contains("timeout") ||
                          error.to_lowercase().contains("timed out") ||
                          error.to_lowercase().contains("connection") ||
                          error.to_lowercase().contains("network");

    // Check if this is a batch recovery scenario
    let is_recovery_scenario = error.contains("interrupted when the app was closed") ||
                              error.contains("resume, restart, or delete");

    html! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
            <div class="text-center">
                <div class="w-16 h-16 mx-auto mb-4 bg-red-100 dark:bg-red-900/20 rounded-full flex items-center justify-center">
                    {if is_recovery_scenario {
                        html! {
                            <svg class="w-8 h-8 text-yellow-600 dark:text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"></path>
                            </svg>
                        }
                    } else {
                        html! {
                            <svg class="w-8 h-8 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                            </svg>
                        }
                    }}
                </div>

                <h3 class="text-xl font-semibold text-gray-800 dark:text-white mb-2">
                    {if is_recovery_scenario { "Batch Recovery" } else { "Batch Failed" }}
                </h3>

                <div class={if is_recovery_scenario { "text-yellow-600 dark:text-yellow-400 mb-4" } else { "text-red-600 dark:text-red-400 mb-4" }}>
                    {error}
                </div>

                {if let Some(batch) = current_batch {
                    html! {
                        <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 mb-6 text-left">
                            <h4 class="text-sm font-semibold text-blue-800 dark:text-blue-300 mb-2">
                                {"Batch Information"}
                            </h4>
                            <div class="text-sm text-blue-700 dark:text-blue-400 space-y-1">
                                <div>{"Name: "}{&batch.name}</div>
                                <div>{"Total Items: "}{batch.items.len()}</div>
                                <div>{"Status: "}{format!("{:?}", batch.status)}</div>
                                {if let Some(completed_at) = &batch.completed_at {
                                    html! {
                                        <div>{"Last Activity: "}{completed_at}</div>
                                    }
                                } else if let Some(started_at) = &batch.started_at {
                                    html! {
                                        <div>{"Started: "}{started_at}</div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}

                {if is_timeout_error {
                    html! {
                        <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4 mb-6 text-left">
                            <h4 class="text-sm font-semibold text-yellow-800 dark:text-yellow-300 mb-2">
                                {"Timeout Issue - This is Common with Facebook"}
                            </h4>
                            <ul class="text-sm text-yellow-700 dark:text-yellow-400 space-y-1">
                                <li>{"• Facebook reel extractions can take 30-60 seconds"}</li>
                                <li>{"• Try reducing concurrent downloads to 1-2 for better stability"}</li>
                                <li>{"• Share links (share/r/...) work faster than reel links"}</li>
                                <li>{"• Check your internet connection and try again"}</li>
                                <li>{"• Consider processing smaller batches (5-10 URLs at a time)"}</li>
                            </ul>
                        </div>
                    }
                } else {
                    html! {}
                }}

                // Recovery action buttons for interrupted batches
                {if is_recovery_scenario && current_batch.is_some() {
                    html! {
                        <div class="flex flex-col sm:flex-row gap-3 justify-center">
                            {if let Some(on_resume) = on_resume_batch {
                                html! {
                                    <button
                                        class="px-6 py-3 bg-green-500 hover:bg-green-600 dark:bg-green-600 dark:hover:bg-green-700 text-white rounded-lg font-medium transition-colors duration-200"
                                        onclick={on_resume}
                                    >
                                        {"▶️ Resume Batch"}
                                    </button>
                                }
                            } else {
                                html! {}
                            }}

                            <button
                                class="px-6 py-3 bg-facebook-500 hover:bg-facebook-600 dark:bg-facebook-600 dark:hover:bg-facebook-700 text-white rounded-lg font-medium transition-colors duration-200"
                                onclick={on_new_batch}
                            >
                                {"🔄 Start New Batch"}
                            </button>

                            {if let Some(on_delete) = on_delete_batch {
                                html! {
                                    <button
                                        class="px-6 py-3 bg-red-500 hover:bg-red-600 dark:bg-red-600 dark:hover:bg-red-700 text-white rounded-lg font-medium transition-colors duration-200"
                                        onclick={on_delete}
                                    >
                                        {"🗑️ Delete Batch"}
                                    </button>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                    }
                } else {
                    html! {
                        <button
                            class="px-6 py-3 bg-facebook-500 hover:bg-facebook-600 dark:bg-facebook-600 dark:hover:bg-facebook-700 text-white rounded-lg font-medium transition-colors duration-200"
                            onclick={on_new_batch}
                        >
                            {"Try Again"}
                        </button>
                    }
                }}
            </div>
        </div>
    }
}


