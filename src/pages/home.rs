use leptos::*;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::console;
use crate::utils::validation::is_valid_facebook_url;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtractionState {
    Idle,
    Extracting,
    Success(String), // Simplified - just store success message for now
    Error(String),
}

#[component]
pub fn HomePage() -> impl IntoView {
    // Simplified state management for now
    let (url_input, set_url_input) = signal(String::new());
    let (extraction_state, set_extraction_state) = signal(ExtractionState::Idle);
    let (url_error, set_url_error) = signal(None::<String>);

    // Handle URL input change
    let on_url_change = move |e: web_sys::Event| {
        let input: web_sys::HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
        let value = input.value();
        set_url_input.set(value);
        set_url_error.set(None);
    };

    // Handle extraction
    let on_extract = move |_| {
        let url = url_input.get();
        console::log_1(&format!("🎯 Extract button clicked with URL: {}", url).into());

        // Basic validation
        if url.is_empty() {
            console::log_1(&"❌ URL is empty".into());
            set_url_error.set(Some("Please enter a Facebook video URL".to_string()));
            return;
        }

        // Comprehensive URL validation using the validation utility
        let is_valid = is_valid_facebook_url(&url);
        console::log_1(&format!("🔍 Frontend URL validation result: {}", is_valid).into());

        if !is_valid {
            console::log_1(&"❌ URL failed frontend validation".into());
            set_url_error.set(Some("Please enter a valid Facebook URL".to_string()));
            return;
        }

        console::log_1(&"✅ URL passed frontend validation".into());
        set_url_error.set(None);
        set_extraction_state.set(ExtractionState::Extracting);

        // For now, just simulate success immediately (we can add delay later)
        set_extraction_state.set(ExtractionState::Success("Video extraction successful! (Demo mode)".to_string()));
    };

    // Handle clear/reset
    let on_clear = move |_| {
        set_url_input.set(String::new());
        set_extraction_state.set(ExtractionState::Idle);
        set_url_error.set(None);
    };



    view! {
        <div class="max-w-4xl mx-auto">
            <div class="text-center mb-8">
                <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">
                    "Facebook Video Downloader 🚀"
                </h1>
                <p class="text-xl text-gray-600 dark:text-gray-400">
                    "Download Facebook videos, Reels, and private content with ease ⚡"
                </p>
            </div>

            <div class="card">
                <div class="card-body">
                    <h2 class="text-2xl font-semibold mb-4">"Extract Video Information"</h2>
                    <p class="text-gray-600 dark:text-gray-400 mb-6">
                        "Paste a Facebook video URL below to get started"
                    </p>

                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                "Facebook Video URL"
                            </label>
                            <div class="flex space-x-3">
                                <div class="relative flex-1">
                                    <input
                                        type="url"
                                        class="input w-full pr-12"
                                        placeholder="https://www.facebook.com/watch?v=..."
                                        prop:value=move || url_input.get()
                                        on:input=on_url_change
                                        prop:disabled=move || matches!(extraction_state.get(), ExtractionState::Extracting)
                                    />
                                </div>
                                <button
                                    class="btn-primary whitespace-nowrap"
                                    on:click=on_extract
                                    prop:disabled=move || matches!(extraction_state.get(), ExtractionState::Extracting) || url_input.get().is_empty()
                                >
                                    {move || if matches!(extraction_state.get(), ExtractionState::Extracting) {
                                        "Extracting..."
                                    } else {
                                        "Extract Video Info"
                                    }}
                                </button>
                            </div>
                            <Show when=move || url_error.get().is_some() fallback=|| ()>
                                <p class="text-red-500 text-sm mt-1">{move || url_error.get().unwrap_or_default()}</p>
                            </Show>
                        </div>

                        <Show when=move || matches!(extraction_state.get(), ExtractionState::Success(_)) fallback=|| ()>
                            <div class="flex justify-end">
                                <button
                                    class="text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 underline transition-colors duration-200"
                                    on:click=on_clear
                                    title="Clear results and extract another video"
                                >
                                    "Extract Another Video"
                                </button>
                            </div>
                        </Show>
                    </div>

                    // Display extraction results
                    {move || match extraction_state.get() {
                        ExtractionState::Success(message) => {
                            view! {
                                <div class="mt-6 p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
                                    <h3 class="text-lg font-semibold text-green-800 dark:text-green-200 mb-2">
                                        "Video Extracted Successfully!"
                                    </h3>
                                    <p class="text-green-600 dark:text-green-300">{message}</p>
                                </div>
                            }.into_any()
                        }
                        ExtractionState::Error(error) => {
                            view! {
                                <div class="mt-6 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                                    <h3 class="text-lg font-semibold text-red-800 dark:text-red-200 mb-2">
                                        "Extraction Failed"
                                    </h3>
                                    <p class="text-red-600 dark:text-red-300">{error}</p>
                                </div>
                            }.into_any()
                        }
                        _ => view! { <div></div> }.into_any()
                    }}
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
                        <h3 class="text-lg font-semibold mb-2">"Regular Videos"</h3>
                        <p class="text-gray-600 dark:text-gray-400 text-sm">
                            "Download public Facebook videos in multiple quality options"
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
                        <h3 class="text-lg font-semibold mb-2">"Facebook Reels"</h3>
                        <p class="text-gray-600 dark:text-gray-400 text-sm">
                            "Download short-form video content from Facebook Reels"
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
                        <h3 class="text-lg font-semibold mb-2">"Private Content"</h3>
                        <p class="text-gray-600 dark:text-gray-400 text-sm">
                            "Download private videos using the view-source method"
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}

