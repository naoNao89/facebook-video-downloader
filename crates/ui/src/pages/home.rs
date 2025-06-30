use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use std::collections::HashMap;
use crate::services::tauri_api::{self, VideoInfo};
use crate::utils::validation::is_valid_facebook_url;
use crate::utils::formatting::{format_duration_time, extract_time_format, format_number};
use crate::components::{PasteIcon, LoadingIcon, PlayIcon, Thumbnail, CompressionQuality, CompressionEstimate, CompressionLevel, CompressionLevelSelector, CompactPrivacyIndicator, CopyButton};
use crate::hooks::use_local_storage::use_local_storage;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtractionState {
    Idle,
    Extracting,
    Success(VideoInfo),
    Error(String),
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HomePageState {
    pub url_input: String,
    pub extraction_state: ExtractionState,
    pub url_error: Option<String>,
    pub download_state: DownloadState,
    pub compression_available: Option<bool>,
    pub compression_estimates: HashMap<usize, CompressionEstimate>,
    pub selected_compression_levels: HashMap<usize, CompressionLevel>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DownloadState {
    Idle,
    Downloading {
        quality_index: usize,
        phase: DownloadPhase,
        progress: Option<String>,
    },
    Success {
        file_path: String,
        original_size_mb: Option<u64>,
        compressed_size_mb: Option<u64>,
        compression_time_seconds: Option<u64>,
    },
    Error(String), // error message
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DownloadPhase {
    DownloadingVideo,
    AnalyzingVideo,
    CompressingVideo { compression_level: CompressionLevel },
    PreservingAudio,
    Finalizing,
}

#[function_component(HomePage)]
pub fn home_page() -> Html {
    // Use persistent state for extraction and download workflow
    let (persistent_state, set_persistent_state) = use_local_storage(
        "facebook_downloader_home_state",
        HomePageState {
            url_input: String::new(),
            extraction_state: ExtractionState::Idle,
            url_error: None,
            download_state: DownloadState::Idle,
            compression_available: None,
            compression_estimates: HashMap::new(),
            selected_compression_levels: HashMap::new(),
        }
    );

    // Extract individual state handles for easier use
    let url_input = use_state(|| persistent_state.url_input.clone());
    let extraction_state = use_state(|| persistent_state.extraction_state.clone());
    let url_error = use_state(|| persistent_state.url_error.clone());
    let download_state = use_state(|| persistent_state.download_state.clone());
    let compression_available = use_state(|| persistent_state.compression_available.clone());
    let compression_estimates = use_state(|| persistent_state.compression_estimates.clone());
    let selected_compression_levels = use_state(|| persistent_state.selected_compression_levels.clone());

    // Non-persistent state (doesn't need to survive navigation)
    let paste_loading = use_state(|| false);

    // Function to update persistent state
    let update_persistent_state = {
        let set_persistent_state = set_persistent_state.clone();
        let url_input = url_input.clone();
        let extraction_state = extraction_state.clone();
        let url_error = url_error.clone();
        let download_state = download_state.clone();
        let compression_available = compression_available.clone();
        let compression_estimates = compression_estimates.clone();
        let selected_compression_levels = selected_compression_levels.clone();

        Callback::from(move |_| {
            let new_state = HomePageState {
                url_input: (*url_input).clone(),
                extraction_state: (*extraction_state).clone(),
                url_error: (*url_error).clone(),
                download_state: (*download_state).clone(),
                compression_available: (*compression_available).clone(),
                compression_estimates: (*compression_estimates).clone(),
                selected_compression_levels: (*selected_compression_levels).clone(),
            };
            set_persistent_state.emit(new_state);
        })
    };

    // Handle URL input change
    let on_url_change = {
        let url_input = url_input.clone();
        let url_error = url_error.clone();
        let update_persistent_state = update_persistent_state.clone();

        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            url_input.set(value);
            url_error.set(None);

            // Update persistent state
            update_persistent_state.emit(());
        })
    };

    // Handle extraction
    let on_extract = {
        let url_input = url_input.clone();
        let extraction_state = extraction_state.clone();
        let url_error = url_error.clone();
        let update_persistent_state = update_persistent_state.clone();

        Callback::from(move |_| {
            let url = (*url_input).clone();
            console::log_1(&format!("🎯 Extract button clicked with URL: {}", url).into());

            // Basic validation
            if url.is_empty() {
                console::log_1(&"❌ URL is empty".into());
                url_error.set(Some("Please enter a Facebook video URL".to_string()));
                return;
            }

            // Comprehensive URL validation using the validation utility
            let is_valid = is_valid_facebook_url(&url);
            console::log_1(&format!("🔍 Frontend URL validation result: {}", is_valid).into());

            if !is_valid {
                console::log_1(&"❌ URL failed frontend validation".into());
                url_error.set(Some("Please enter a valid Facebook URL".to_string()));
                return;
            }

            console::log_1(&"✅ URL passed frontend validation, proceeding to Tauri backend".into());

            url_error.set(None);
            extraction_state.set(ExtractionState::Extracting);
            update_persistent_state.emit(());

            let extraction_state = extraction_state.clone();
            let update_persistent_state = update_persistent_state.clone();

            spawn_local(async move {
                console::log_1(&"🚀 Testing Tauri API connection...".into());

                // First test if Tauri API is available
                match tauri_api::test_tauri_connection().await {
                    Ok(message) => {
                        console::log_1(&format!("✅ Tauri API test successful: {}", message).into());
                        console::log_1(&"🚀 Starting Facebook video extraction via Tauri backend".into());
                    }
                    Err(e) => {
                        console::log_1(&format!("❌ Tauri API test failed: {}", e.error).into());
                        extraction_state.set(ExtractionState::Error(e.error));
                        update_persistent_state.emit(());
                        return;
                    }
                }

                // Now validate the URL using Tauri backend
                match tauri_api::validate_facebook_url(url.clone()).await {
                    Ok(validation) => {
                        if !validation.is_valid {
                            let error_msg = validation.error_message
                                .unwrap_or_else(|| "Invalid Facebook URL format".to_string());
                            console::log_1(&format!("❌ URL validation failed: {}", error_msg).into());
                            extraction_state.set(ExtractionState::Error(error_msg));
                            update_persistent_state.emit(());
                            return;
                        }

                        // URL is valid, proceed with extraction
                        console::log_1(&"✅ URL validation passed, extracting video info...".into());
                        console::log_1(&format!("🔍 [FRONTEND] About to call tauri_api::extract_video_info with URL: {}", url).into());

                        match tauri_api::extract_video_info(url.clone()).await {
                            Ok(video_info) => {
                                console::log_1(&"✅ [FRONTEND] Extraction completed successfully!".into());
                                console::log_1(&format!("✅ [FRONTEND] Received video info - Title: {}", video_info.title).into());
                                console::log_1(&format!("✅ [FRONTEND] Video has {} quality options", video_info.qualities.len()).into());
                                console::log_1(&"🔄 [FRONTEND] Setting extraction state to Success...".into());
                                extraction_state.set(ExtractionState::Success(video_info));
                                update_persistent_state.emit(());
                                console::log_1(&"✅ [FRONTEND] Extraction state updated to Success".into());
                            }
                            Err(e) => {
                                console::log_1(&format!("❌ [FRONTEND] Extraction failed: {}", e.error).into());
                                console::log_1(&format!("❌ [FRONTEND] Error details: {:?}", e).into());
                                extraction_state.set(ExtractionState::Error(e.error));
                                update_persistent_state.emit(());
                            }
                        }
                    }
                    Err(e) => {
                        // Handle Tauri API not available or other validation errors
                        console::log_1(&format!("❌ URL validation failed: {}", e.error).into());
                        extraction_state.set(ExtractionState::Error(e.error));
                        update_persistent_state.emit(());
                    }
                }
            });
        })
    };

    // Handle paste from clipboard
    let on_paste = {
        let url_input = url_input.clone();
        let url_error = url_error.clone();
        let paste_loading = paste_loading.clone();

        Callback::from(move |_| {
            let url_input = url_input.clone();
            let url_error = url_error.clone();
            let paste_loading = paste_loading.clone();

            paste_loading.set(true);
            url_error.set(None);

            spawn_local(async move {
                match tauri_api::read_clipboard().await {
                    Ok(clipboard_content) => {
                        console::log_1(&format!("📋 Clipboard content: {}", clipboard_content).into());

                        if clipboard_content.trim().is_empty() {
                            url_error.set(Some("Clipboard is empty".to_string()));
                        } else if is_valid_facebook_url(&clipboard_content) {
                            url_input.set(clipboard_content);
                            console::log_1(&"✅ Valid Facebook URL pasted from clipboard".into());
                        } else {
                            url_error.set(Some("Clipboard doesn't contain a valid Facebook URL".to_string()));
                        }
                    }
                    Err(e) => {
                        console::log_1(&format!("❌ Failed to read clipboard: {}", e.error).into());
                        url_error.set(Some("Failed to read clipboard. Please paste manually.".to_string()));
                    }
                }
                paste_loading.set(false);
            });
        })
    };

    // Handle clear/reset
    let on_clear = {
        let url_input = url_input.clone();
        let extraction_state = extraction_state.clone();
        let url_error = url_error.clone();
        let download_state = download_state.clone();
        let update_persistent_state = update_persistent_state.clone();

        Callback::from(move |_| {
            url_input.set(String::new());
            extraction_state.set(ExtractionState::Idle);
            url_error.set(None);
            download_state.set(DownloadState::Idle);

            // Update persistent state
            update_persistent_state.emit(());
        })
    };

    // Check compression availability on mount
    {
        let compression_available = compression_available.clone();
        let update_persistent_state = update_persistent_state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match tauri_api::check_compression_availability().await {
                    Ok(result) => {
                        compression_available.set(Some(result.data));
                        update_persistent_state.emit(());
                        console::log_1(&format!("✅ Compression availability: {}", result.data).into());
                    }
                    Err(e) => {
                        console::log_1(&format!("❌ Failed to check compression availability: {}", e.error).into());
                        compression_available.set(Some(false));
                        update_persistent_state.emit(());
                    }
                }
            });
            || ()
        });
    }

    html! {
        <div class="max-w-4xl mx-auto">
            <div class="text-center mb-8">
                <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">
                    {"Facebook Video Downloader 🚀"}
                </h1>
                <p class="text-xl text-gray-600 dark:text-gray-400">
                    {"Download Facebook videos, Reels, and private content with ease ⚡"}
                </p>
            </div>

            <div class="card">
                <div class="card-body">
                    <h2 class="text-2xl font-semibold mb-4">{"Extract Video Information"}</h2>
                    <p class="text-gray-600 dark:text-gray-400 mb-6">
                        {"Paste a Facebook video URL below to get started"}
                    </p>

                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                {"Facebook Video URL"}
                            </label>
                            <div class="flex space-x-3">
                                <div class="relative flex-1">
                                    <input
                                        type="url"
                                        class="input w-full pr-12"
                                        placeholder="https://www.facebook.com/watch?v=..."
                                        value={(*url_input).clone()}
                                        onchange={on_url_change}
                                        disabled={matches!(*extraction_state, ExtractionState::Extracting)}
                                    />
                                    <div class="absolute inset-y-0 right-0 flex items-center pr-3">
                                        <button
                                            class="inline-flex items-center justify-center p-1 text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300 rounded transition-colors duration-200 hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-1"
                                            onclick={on_paste}
                                            disabled={*paste_loading || matches!(*extraction_state, ExtractionState::Extracting)}
                                            title="Paste from clipboard"
                                            aria-label="Paste from clipboard"
                                        >
                                            {
                                                if *paste_loading {
                                                    html! {
                                                        <LoadingIcon size="4" class="" />
                                                    }
                                                } else {
                                                    html! {
                                                        <PasteIcon size="4" class="" />
                                                    }
                                                }
                                            }
                                        </button>
                                    </div>
                                </div>
                                <button
                                    class="btn-primary whitespace-nowrap"
                                    onclick={on_extract}
                                    disabled={matches!(*extraction_state, ExtractionState::Extracting) || url_input.is_empty()}
                                >
                                    {
                                        if matches!(*extraction_state, ExtractionState::Extracting) {
                                            "Extracting..."
                                        } else {
                                            "Extract Video Info"
                                        }
                                    }
                                </button>
                            </div>
                            {
                                if let Some(error) = &*url_error {
                                    html! {
                                        <p class="text-red-500 text-sm mt-1">{error}</p>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </div>

                        {
                            if matches!(*extraction_state, ExtractionState::Success(_)) {
                                html! {
                                    <div class="flex justify-end">
                                        <button
                                            class="text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 underline transition-colors duration-200"
                                            onclick={on_clear}
                                            title="Clear results and extract another video"
                                        >
                                            {"Extract Another Video"}
                                        </button>
                                    </div>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>

                    // Display extraction results
                    {
                        match &*extraction_state {
                            ExtractionState::Success(video_info) => {
                                render_video_info(
                                    video_info.clone(),
                                    download_state.clone(),
                                    compression_available.clone(),
                                    compression_estimates.clone(),
                                    selected_compression_levels.clone(),
                                    update_persistent_state.clone(),
                                )
                            }
                            ExtractionState::Error(error) => {
                                html! {
                                    <div class="mt-6 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                                        <h3 class="text-lg font-semibold text-red-800 dark:text-red-200 mb-2">
                                            {"Extraction Failed"}
                                        </h3>
                                        <p class="text-red-600 dark:text-red-300">{error}</p>
                                    </div>
                                }
                            }
                            _ => html! {}
                        }
                    }
                </div>
            </div>
            
            <div class="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="card">
                    <div class="card-body text-center">
                        <div class="w-12 h-12 bg-facebook-100 dark:bg-facebook-900 rounded-lg flex items-center justify-center mx-auto mb-4">
                            <svg class="w-6 h-6 text-facebook-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4V2a1 1 0 011-1h8a1 1 0 011 1v2m0 0V1a1 1 0 011 1v8a1 1 0 01-1 1M7 4H5a1 1 0 00-1 1v8a1 1 0 001 1h2m0-10h8m-8 0V1a1 1 0 011-1h6a1 1 0 011 1v3"></path>
                            </svg>
                        </div>
                        <h3 class="text-lg font-semibold mb-2">{"Regular Videos"}</h3>
                        <p class="text-gray-600 dark:text-gray-400 text-sm">
                            {"Download public Facebook videos in multiple quality options"}
                        </p>
                    </div>
                </div>
                
                <div class="card">
                    <div class="card-body text-center">
                        <div class="w-12 h-12 bg-success-100 dark:bg-success-900 rounded-lg flex items-center justify-center mx-auto mb-4">
                            <svg class="w-6 h-6 text-success-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"></path>
                            </svg>
                        </div>
                        <h3 class="text-lg font-semibold mb-2">{"Facebook Reels"}</h3>
                        <p class="text-gray-600 dark:text-gray-400 text-sm">
                            {"Download short-form video content from Facebook Reels"}
                        </p>
                    </div>
                </div>
                
                <div class="card">
                    <div class="card-body text-center">
                        <div class="w-12 h-12 bg-warning-100 dark:bg-warning-900 rounded-lg flex items-center justify-center mx-auto mb-4">
                            <svg class="w-6 h-6 text-warning-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path>
                            </svg>
                        </div>
                        <h3 class="text-lg font-semibold mb-2">{"Private Content"}</h3>
                        <p class="text-gray-600 dark:text-gray-400 text-sm">
                            {"Download private videos using the view-source method"}
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}



/// Render duration badge for thumbnail overlay
fn render_duration_badge(video_info: &VideoInfo) -> Html {
    if let Some(seconds) = video_info.metadata.duration_seconds {
        let time_format = format_duration_time(seconds);
        html! {
            <div class="absolute bottom-2 right-2 bg-black bg-opacity-75 text-white text-xs px-2 py-1 rounded">
                {time_format}
            </div>
        }
    } else if video_info.duration != "Unknown duration" {
        let time_format = extract_time_format(&video_info.duration);
        if time_format != "Unknown" {
            html! {
                <div class="absolute bottom-2 right-2 bg-black bg-opacity-75 text-white text-xs px-2 py-1 rounded">
                    {time_format}
                </div>
            }
        } else {
            html! {}
        }
    } else {
        html! {}
    }
}

/// Render video information display
fn render_video_info(
    video_info: VideoInfo,
    download_state: UseStateHandle<DownloadState>,
    compression_available: UseStateHandle<Option<bool>>,
    compression_estimates: UseStateHandle<HashMap<usize, CompressionEstimate>>,
    selected_compression_levels: UseStateHandle<HashMap<usize, CompressionLevel>>,
    update_persistent_state: Callback<()>,
) -> Html {
    html! {
        <div class="mt-6 space-y-6">
            // Success alert
            <div class="p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                <h3 class="text-lg font-semibold text-green-800 dark:text-green-200 mb-2">
                    {"Video Extracted Successfully!"}
                </h3>
                <p class="text-green-600 dark:text-green-300">
                    {format!("Found {} quality options for download", video_info.qualities.len())}
                </p>
            </div>

            // Video metadata with thumbnail
            <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-6">
                <h3 class="text-xl font-semibold mb-4">{"Video Information"}</h3>
                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    // Thumbnail section
                    <div class="lg:col-span-1">
                        <div class="aspect-video bg-gray-200 dark:bg-gray-700 rounded-lg overflow-hidden relative group">
                            <Thumbnail
                                url={video_info.thumbnail.clone()}
                                alt="Video thumbnail"
                                class="rounded-lg"
                            />
                            // Duration badge (bottom-right corner)
                            { render_duration_badge(&video_info) }

                            // Play overlay - only show if thumbnail is loaded (including SVG fallbacks)
                            {
                                if !video_info.thumbnail.is_empty() {
                                    html! {
                                        <div class="absolute inset-0 bg-black bg-opacity-30 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                                            <div class="bg-white bg-opacity-90 rounded-full p-3">
                                                <PlayIcon size="8" class="text-gray-800" />
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </div>


                    </div>

                    // Video details section
                    <div class="lg:col-span-2">
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div class="md:col-span-2">
                                <p class="text-sm text-gray-600 dark:text-gray-400">{"Title"}</p>
                                <p class="font-medium text-lg">{&video_info.title}</p>
                            </div>

                            <div>
                                <p class="text-sm text-gray-600 dark:text-gray-400">{"Author"}</p>
                                <div class="flex items-center space-x-2">
                                    <p class="font-medium">{&video_info.metadata.author}</p>
                                    <CompactPrivacyIndicator
                                        privacy_level={video_info.privacy_level.clone()}
                                        show_label={false}
                                    />
                                </div>
                            </div>
                            <div>
                                <p class="text-sm text-gray-600 dark:text-gray-400">{"Views"}</p>
                                <p class="font-medium">{format_number(video_info.metadata.views)}</p>
                            </div>
                            <div>
                                <p class="text-sm text-gray-600 dark:text-gray-400">{"Video ID"}</p>
                                <div class="flex items-center space-x-2">
                                    <p class="font-medium font-mono text-sm">{&video_info.video_id}</p>
                                    <CopyButton
                                        text={video_info.video_id.clone()}
                                        size="4"
                                        tooltip="Copy Video ID"
                                    />
                                </div>
                            </div>
                            <div>
                                <p class="text-sm text-gray-600 dark:text-gray-400">{"Extraction Time"}</p>
                                <p class="font-medium text-sm">{&video_info.extraction_timestamp}</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            // Quality options
            <div class="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700">
                <div class="p-6 border-b border-gray-200 dark:border-gray-700">
                    <h3 class="text-xl font-semibold">{"Available Quality Options"}</h3>
                    <p class="text-gray-600 dark:text-gray-400 mt-1">
                        {"Choose a quality to download the video"}
                    </p>
                </div>
                <div class="divide-y divide-gray-200 dark:divide-gray-700">
                    {
                        video_info.qualities.iter().enumerate().map(|(index, quality)| {
                            let quality_clone = quality.clone();
                            let compression_available_clone = compression_available.clone();
                            let compression_estimates_clone = compression_estimates.clone();
                            let selected_compression_levels_clone = selected_compression_levels.clone();

                            // Get the selected compression level for this quality (default to Original)
                            let selected_level = selected_compression_levels.get(&index)
                                .copied()
                                .unwrap_or(CompressionLevel::Original);

                            // Calculate display size based on selected compression level
                            let display_size = if selected_level == CompressionLevel::Original {
                                format!("~{} MB", quality.estimated_size_mb)
                            } else if let Some(estimate) = compression_estimates.get(&index) {
                                format!("~{} MB", estimate.estimated_size_mb)
                            } else {
                                // Calculate estimate based on compression ratio
                                if let Some(compression_quality) = selected_level.to_compression_quality() {
                                    let ratio = match compression_quality {
                                        CompressionQuality::High => 0.85,
                                        CompressionQuality::Medium => 0.60,
                                        CompressionQuality::Low => 0.40,
                                        CompressionQuality::Minimal => 0.25,
                                    };
                                    let estimated_size = (quality.estimated_size_mb as f64 * ratio) as u64;
                                    format!("~{} MB", estimated_size)
                                } else {
                                    format!("~{} MB", quality.estimated_size_mb)
                                }
                            };

                            html! {
                                <div class="p-6 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors">
                                    <div class="flex items-center justify-between">
                                        <div class="flex-1">
                                            <div class="flex items-center space-x-4 mb-2">
                                                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
                                                    {&quality.quality}
                                                </span>
                                                <span class="text-sm text-gray-600 dark:text-gray-400">
                                                    {format!("{}x{}", quality.width, quality.height)}
                                                </span>
                                                <span class="text-sm text-gray-600 dark:text-gray-400">
                                                    {&quality.format}
                                                </span>
                                                <span class="text-sm text-gray-600 dark:text-gray-400">
                                                    {display_size}
                                                </span>
                                            </div>
                                        </div>
                                        <div class="flex items-center space-x-3">
                                            <button
                                                class="btn-secondary btn-small"
                                                onclick={
                                                    let url = quality_clone.download_url.clone();
                                                    Callback::from(move |_| {
                                                        // Copy URL to clipboard - simplified for now
                                                        web_sys::console::log_1(&format!("Copy URL: {}", url).into());
                                                    })
                                                }
                                            >
                                                {"Copy URL"}
                                            </button>

                                            // Compression level selector
                                            <div class="min-w-[120px]">
                                                <CompressionLevelSelector
                                                    original_size_mb={quality.estimated_size_mb as u64}
                                                    selected_level={selected_level}
                                                    compression_available={compression_available_clone.unwrap_or(false)}
                                                    on_level_change={
                                                        let selected_compression_levels = selected_compression_levels_clone.clone();
                                                        let compression_estimates = compression_estimates_clone.clone();
                                                        let quality_index = index;
                                                        let quality_size = quality.estimated_size_mb as u64;
                                                        let update_persistent_state_clone = update_persistent_state.clone();

                                                        Callback::from(move |level: CompressionLevel| {
                                                            // Update selected compression level
                                                            let mut levels = (*selected_compression_levels).clone();
                                                            levels.insert(quality_index, level);
                                                            selected_compression_levels.set(levels);
                                                            update_persistent_state_clone.emit(());

                                                            // If compression is selected, get estimate
                                                            if let Some(compression_quality) = level.to_compression_quality() {
                                                                let compression_estimates = compression_estimates.clone();
                                                                let quality_index = quality_index;
                                                                let quality_size = quality_size;
                                                                let update_persistent_state_async = update_persistent_state_clone.clone();

                                                                spawn_local(async move {
                                                                    console::log_1(&format!("🗜️ Estimating compression for quality index: {} with {:?}", quality_index, compression_quality).into());

                                                                    match tauri_api::estimate_compression_size(quality_size, compression_quality).await {
                                                                        Ok(result) => {
                                                                            console::log_1(&format!("✅ Compression estimate: {} MB -> {} MB", quality_size, result.data.estimated_size_mb).into());
                                                                            let mut estimates = (*compression_estimates).clone();
                                                                            estimates.insert(quality_index, result.data);
                                                                            compression_estimates.set(estimates);
                                                                            update_persistent_state_async.emit(());
                                                                        }
                                                                        Err(e) => {
                                                                            console::log_1(&format!("❌ Compression estimate failed: {}", e.error).into());
                                                                        }
                                                                    }
                                                                });
                                                            }
                                                        })
                                                    }
                                                />
                                            </div>
                                            <button
                                                class={format!("btn-primary btn-small {}",
                                                    if matches!(*download_state, DownloadState::Downloading { quality_index, .. } if quality_index == index) {
                                                        "opacity-50 cursor-not-allowed"
                                                    } else {
                                                        ""
                                                    }
                                                )}
                                                onclick={
                                                    let video_info_clone = video_info.clone();
                                                    let download_state = download_state.clone();
                                                    let selected_compression_levels = selected_compression_levels_clone.clone();
                                                    let quality_index = index;

                                                    let update_persistent_state_clone = update_persistent_state.clone();
                                                    Callback::from(move |_| {
                                                        let video_info = video_info_clone.clone();
                                                        let download_state = download_state.clone();
                                                        let selected_level = selected_compression_levels.get(&quality_index)
                                                            .copied()
                                                            .unwrap_or(CompressionLevel::Original);

                                                        // Start with downloading phase
                                                        download_state.set(DownloadState::Downloading {
                                                            quality_index,
                                                            phase: DownloadPhase::DownloadingVideo,
                                                            progress: Some("Initializing download...".to_string()),
                                                        });
                                                        update_persistent_state_clone.emit(());

                                                        let update_persistent_state_async = update_persistent_state_clone.clone();
                                                        spawn_local(async move {
                                                            console::log_1(&format!("🚀 Starting download for quality index: {} with compression level: {:?}", quality_index, selected_level).into());

                                                            // Update progress: downloading video
                                                            download_state.set(DownloadState::Downloading {
                                                                quality_index,
                                                                phase: DownloadPhase::DownloadingVideo,
                                                                progress: Some("Downloading original video...".to_string()),
                                                            });
                                                            update_persistent_state_async.emit(());

                                                            // First download the original video
                                                            match tauri_api::download_video(video_info.clone(), quality_index, None).await {
                                                                Ok(file_path) => {
                                                                    let original_size_mb = if let Ok(metadata) = std::fs::metadata(&file_path) {
                                                                        Some(metadata.len() / 1024 / 1024)
                                                                    } else {
                                                                        None
                                                                    };

                                                                    // If compression is requested, compress the downloaded file
                                                                    if let Some(compression_quality) = selected_level.to_compression_quality() {
                                                                        console::log_1(&format!("🗜️ Starting compression with {:?} quality", compression_quality).into());

                                                                        // Update progress: analyzing video
                                                                        download_state.set(DownloadState::Downloading {
                                                                            quality_index,
                                                                            phase: DownloadPhase::AnalyzingVideo,
                                                                            progress: Some("Analyzing video for accurate compression estimation...".to_string()),
                                                                        });
                                                                        update_persistent_state_async.emit(());

                                                                        // Get accurate compression estimate now that we have the file
                                                                        if let Ok(estimate_result) = tauri_api::estimate_compression_size_from_file(file_path.clone(), compression_quality).await {
                                                                            console::log_1(&format!("📊 Accurate estimate: {} MB -> {} MB",
                                                                                estimate_result.data.original_size_mb,
                                                                                estimate_result.data.estimated_size_mb).into());
                                                                        }

                                                                        // Update progress: compressing video
                                                                        download_state.set(DownloadState::Downloading {
                                                                            quality_index,
                                                                            phase: DownloadPhase::CompressingVideo { compression_level: selected_level },
                                                                            progress: Some(format!("Compressing video with {}% quality...", selected_level.percentage())),
                                                                        });
                                                                        update_persistent_state_async.emit(());

                                                                        // Generate compressed file path
                                                                        let compressed_path = file_path.replace(".mp4", &format!("_{}%.mp4", selected_level.percentage()));

                                                                        let compression_start = web_sys::js_sys::Date::now();

                                                                        // Use preserve_original=false to automatically clean up the original file
                                                                        match tauri_api::compress_video(file_path.clone(), compressed_path.clone(), compression_quality, false).await {
                                                                            Ok(result) => {
                                                                                let compression_time = ((web_sys::js_sys::Date::now() - compression_start) / 1000.0) as u64;

                                                                                // Update progress: preserving audio
                                                                                download_state.set(DownloadState::Downloading {
                                                                                    quality_index,
                                                                                    phase: DownloadPhase::PreservingAudio,
                                                                                    progress: Some("Verifying audio streams...".to_string()),
                                                                                });
                                                                                update_persistent_state_async.emit(());

                                                                                // Update progress: finalizing
                                                                                download_state.set(DownloadState::Downloading {
                                                                                    quality_index,
                                                                                    phase: DownloadPhase::Finalizing,
                                                                                    progress: Some("Finalizing compressed video...".to_string()),
                                                                                });
                                                                                update_persistent_state_async.emit(());

                                                                                console::log_1(&format!("✅ Download and compression completed: {}", compressed_path).into());
                                                                                download_state.set(DownloadState::Success {
                                                                                    file_path: compressed_path,
                                                                                    original_size_mb,
                                                                                    compressed_size_mb: Some(result.data.compressed_size_mb),
                                                                                    compression_time_seconds: Some(compression_time),
                                                                                });
                                                                                update_persistent_state_async.emit(());
                                                                            }
                                                                            Err(e) => {
                                                                                console::log_1(&format!("❌ Compression failed: {}", e.error).into());
                                                                                // Still show success with original file if compression fails
                                                                                download_state.set(DownloadState::Success {
                                                                                    file_path: format!("{} (compression failed: {})", file_path, e.error),
                                                                                    original_size_mb,
                                                                                    compressed_size_mb: None,
                                                                                    compression_time_seconds: None,
                                                                                });
                                                                                update_persistent_state_async.emit(());
                                                                            }
                                                                        }
                                                                    } else {
                                                                        console::log_1(&format!("✅ Download completed: {}", file_path).into());
                                                                        download_state.set(DownloadState::Success {
                                                                            file_path,
                                                                            original_size_mb,
                                                                            compressed_size_mb: None,
                                                                            compression_time_seconds: None,
                                                                        });
                                                                        update_persistent_state_async.emit(());
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    console::log_1(&format!("❌ Download failed: {}", e.error).into());
                                                                    download_state.set(DownloadState::Error(e.error));
                                                                    update_persistent_state_async.emit(());
                                                                }
                                                            }
                                                        });
                                                    })
                                                }
                                                disabled={matches!(*download_state, DownloadState::Downloading { .. })}
                                            >
                                                {
                                                    match &*download_state {
                                                        DownloadState::Downloading { quality_index: dl_index, phase, .. } if *dl_index == index => {
                                                            match phase {
                                                                DownloadPhase::DownloadingVideo => "Downloading...",
                                                                DownloadPhase::AnalyzingVideo => "Analyzing...",
                                                                DownloadPhase::CompressingVideo { .. } => "Compressing...",
                                                                DownloadPhase::PreservingAudio => "Preserving Audio...",
                                                                DownloadPhase::Finalizing => "Finalizing...",
                                                            }
                                                        }
                                                        _ => {
                                                            if selected_level.is_compressed() {
                                                                "Download & Compress"
                                                            } else {
                                                                "Download"
                                                            }
                                                        }
                                                    }
                                                }
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </div>

            // Enhanced download status display
            {
                match &*download_state {
                    DownloadState::Downloading { quality_index, phase, progress } => {
                        let phase_description = match phase {
                            DownloadPhase::DownloadingVideo => "Downloading Video",
                            DownloadPhase::AnalyzingVideo => "Analyzing Video Streams",
                            DownloadPhase::CompressingVideo { compression_level } => {
                                &format!("Compressing with {}% Quality", compression_level.percentage())
                            },
                            DownloadPhase::PreservingAudio => "Preserving Audio Streams",
                            DownloadPhase::Finalizing => "Finalizing Process",
                        };

                        let phase_icon = match phase {
                            DownloadPhase::DownloadingVideo => "⬇️",
                            DownloadPhase::AnalyzingVideo => "🔍",
                            DownloadPhase::CompressingVideo { .. } => "🗜️",
                            DownloadPhase::PreservingAudio => "🎵",
                            DownloadPhase::Finalizing => "✨",
                        };

                        html! {
                            <div class="p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                                <div class="flex items-center space-x-3 mb-3">
                                    <span class="text-2xl">{phase_icon}</span>
                                    <div>
                                        <h3 class="text-lg font-semibold text-blue-800 dark:text-blue-200">
                                            {phase_description}
                                        </h3>
                                        <p class="text-sm text-blue-600 dark:text-blue-300">
                                            {format!("Quality: {}",
                                                if let Some(quality) = video_info.qualities.get(*quality_index) {
                                                    &quality.quality
                                                } else {
                                                    "Unknown"
                                                }
                                            )}
                                        </p>
                                    </div>
                                </div>
                                {
                                    if let Some(progress_text) = progress {
                                        html! {
                                            <div class="bg-blue-100 dark:bg-blue-800 p-2 rounded text-sm text-blue-700 dark:text-blue-200">
                                                {progress_text}
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                                <div class="mt-3 bg-blue-200 dark:bg-blue-700 rounded-full h-2">
                                    <div class="bg-blue-600 dark:bg-blue-400 h-2 rounded-full animate-pulse" style="width: 60%"></div>
                                </div>
                            </div>
                        }
                    }
                    DownloadState::Success { file_path, original_size_mb, compressed_size_mb, compression_time_seconds } => {
                        html! {
                            <div class="p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                                <div class="flex items-center space-x-3 mb-3">
                                    <span class="text-2xl">{"✅"}</span>
                                    <h3 class="text-lg font-semibold text-green-800 dark:text-green-200">
                                        {"Download Completed!"}
                                    </h3>
                                </div>

                                <div class="space-y-2 mb-3">
                                    <p class="text-green-600 dark:text-green-300">
                                        {"Video downloaded successfully with audio preserved"}
                                    </p>

                                    // File size information
                                    {
                                        if let (Some(original), Some(compressed)) = (original_size_mb, compressed_size_mb) {
                                            let savings = if *original > *compressed {
                                                *original - *compressed
                                            } else {
                                                0
                                            };
                                            let savings_percent = if *original > 0 {
                                                (savings as f64 / *original as f64 * 100.0) as u32
                                            } else {
                                                0
                                            };

                                            html! {
                                                <div class="bg-green-100 dark:bg-green-800 p-2 rounded text-sm">
                                                    <div class="flex justify-between items-center">
                                                        <span class="text-green-700 dark:text-green-200">{"Original size:"}</span>
                                                        <span class="font-mono text-green-800 dark:text-green-100">{format!("{} MB", original)}</span>
                                                    </div>
                                                    <div class="flex justify-between items-center">
                                                        <span class="text-green-700 dark:text-green-200">{"Compressed size:"}</span>
                                                        <span class="font-mono text-green-800 dark:text-green-100">{format!("{} MB", compressed)}</span>
                                                    </div>
                                                    <div class="flex justify-between items-center font-semibold">
                                                        <span class="text-green-700 dark:text-green-200">{"Space saved:"}</span>
                                                        <span class="font-mono text-green-800 dark:text-green-100">{format!("{} MB ({}%)", savings, savings_percent)}</span>
                                                    </div>
                                                    {
                                                        if let Some(time) = compression_time_seconds {
                                                            html! {
                                                                <div class="flex justify-between items-center mt-1 pt-1 border-t border-green-200 dark:border-green-700">
                                                                    <span class="text-green-700 dark:text-green-200">{"Compression time:"}</span>
                                                                    <span class="font-mono text-green-800 dark:text-green-100">{format!("{}s", time)}</span>
                                                                </div>
                                                            }
                                                        } else {
                                                            html! {}
                                                        }
                                                    }
                                                </div>
                                            }
                                        } else if let Some(original) = original_size_mb {
                                            html! {
                                                <div class="bg-green-100 dark:bg-green-800 p-2 rounded text-sm">
                                                    <div class="flex justify-between items-center">
                                                        <span class="text-green-700 dark:text-green-200">{"File size:"}</span>
                                                        <span class="font-mono text-green-800 dark:text-green-100">{format!("{} MB", original)}</span>
                                                    </div>
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>

                                <p class="text-sm font-mono bg-green-100 dark:bg-green-800 p-2 rounded break-all text-green-800 dark:text-green-100">
                                    {file_path}
                                </p>
                            </div>
                        }
                    }
                    DownloadState::Error(error) => {
                        html! {
                            <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                                <div class="flex items-center space-x-3 mb-2">
                                    <span class="text-2xl">{"❌"}</span>
                                    <h3 class="text-lg font-semibold text-red-800 dark:text-red-200">
                                        {"Download Failed"}
                                    </h3>
                                </div>
                                <p class="text-red-600 dark:text-red-300">{error}</p>
                            </div>
                        }
                    }
                    _ => html! {}
                }
            }
        </div>
    }
}
