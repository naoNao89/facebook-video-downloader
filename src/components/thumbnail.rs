use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::tauri_api;
use crate::components::icons::{ImagePlaceholderIcon, LoadingIcon};

#[derive(Clone, PartialEq, Copy)]
enum ThumbnailState {
    Loading,
    Loaded,
    Error,
    Empty,
}

#[component]
pub fn Thumbnail(
    url: String,
    alt: String,
    #[prop(default = String::new())] class: String,
) -> impl IntoView {
    let (thumbnail_state, set_thumbnail_state) = signal(
        if url.is_empty() || url.contains("placeholder") {
            ThumbnailState::Empty
        } else {
            ThumbnailState::Loading
        }
    );
    let (data_url, set_data_url) = signal(String::new());

    // Effect to fetch thumbnail when URL changes
    Effect::new(move |_| {
        let url = url.clone();
        web_sys::console::log_1(&format!("🔍 [Thumbnail] Processing thumbnail URL (length: {})", url.len()).into());
        web_sys::console::log_1(&format!("🔍 [Thumbnail] URL type: {}",
            if url.is_empty() { "empty" }
            else if url.contains("placeholder") { "placeholder" }
            else if url.starts_with("data:") { "data URL" }
            else if url.starts_with("http") { "HTTP URL" }
            else { "unknown" }
        ).into());

        if url.is_empty() || url.contains("placeholder") {
            web_sys::console::log_1(&"📊 [Thumbnail] Thumbnail status: Empty (empty/placeholder URL)".into());
            set_thumbnail_state.set(ThumbnailState::Empty);
            return;
        }

        web_sys::console::log_1(&"🖼️ Setting state to Loading".into());
        set_thumbnail_state.set(ThumbnailState::Loading);

        // Decode HTML entities in the URL
        let decoded_url = url
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");

        web_sys::console::log_1(&format!("🖼️ Processing thumbnail URL (length: {}, type: {})",
            decoded_url.len(),
            if decoded_url.starts_with("data:") { "data URL" } else { "HTTP URL" }
        ).into());

        // Check if the URL is already a data URL (from fallback SVG)
        if decoded_url.starts_with("data:") {
            web_sys::console::log_1(&"🖼️ [Thumbnail] Using data URL for thumbnail display".into());

            // Validate data URL format
            if decoded_url.starts_with("data:image/") {
                web_sys::console::log_1(&"📊 [Thumbnail] Thumbnail status: Valid data URL (image)".into());
                set_data_url.set(decoded_url);
                set_thumbnail_state.set(ThumbnailState::Loaded);
            } else if decoded_url.starts_with("data:") {
                web_sys::console::log_1(&"📊 [Thumbnail] Thumbnail status: Valid data URL (non-image)".into());
                set_data_url.set(decoded_url);
                set_thumbnail_state.set(ThumbnailState::Loaded);
            } else {
                web_sys::console::error_1(&"❌ Invalid data URL format - setting to Error".into());
                web_sys::console::log_1(&"📊 [Thumbnail] Thumbnail status: Error (invalid data URL)".into());
                set_thumbnail_state.set(ThumbnailState::Error);
            }
            return;
        }

        // For HTTP URLs, fetch through Tauri API
        spawn_local(async move {
            web_sys::console::log_1(&"🚀 Fetching thumbnail via Tauri API...".into());
            web_sys::console::log_1(&format!("🔍 HTTP URL: {}", decoded_url).into());

            match tauri_api::fetch_thumbnail_image(decoded_url.clone()).await {
                Ok(fetched_data_url) => {
                    web_sys::console::log_1(&format!("✅ Thumbnail fetched successfully! Data URL length: {}", fetched_data_url.len()).into());

                    // Test if the data URL is valid
                    if fetched_data_url.starts_with("data:image/") {
                        web_sys::console::log_1(&"✅ Data URL format is valid".into());
                        web_sys::console::log_1(&"🎯 Setting thumbnail state to Loaded".into());
                        set_data_url.set(fetched_data_url);
                        set_thumbnail_state.set(ThumbnailState::Loaded);
                    } else if fetched_data_url.starts_with("data:") {
                        web_sys::console::log_1(&"✅ Data URL format detected (non-image)".into());
                        web_sys::console::log_1(&"🎯 Setting thumbnail state to Loaded (placeholder)".into());
                        set_data_url.set(fetched_data_url);
                        set_thumbnail_state.set(ThumbnailState::Loaded);
                    } else {
                        web_sys::console::error_1(&"❌ Invalid data URL format received".into());
                        web_sys::console::log_1(&"🎯 Setting thumbnail state to Error due to invalid format".into());
                        set_thumbnail_state.set(ThumbnailState::Error);
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("❌ Failed to fetch thumbnail from URL: {}", decoded_url).into());
                    web_sys::console::error_1(&format!("❌ Error details: {:?}", e).into());
                    web_sys::console::log_1(&"🎯 Setting thumbnail state to Error due to fetch failure".into());
                    set_thumbnail_state.set(ThumbnailState::Error);
                }
            }
        });
    });

    let base_class = format!("w-full h-full {}", class);

    view! {
        {move || match thumbnail_state.get() {
            ThumbnailState::Loading => {
                view! {
                    <div class=format!("{} bg-gray-200 dark:bg-gray-700 flex items-center justify-center", base_class)>
                        <div class="flex flex-col items-center space-y-2">
                            <LoadingIcon size="8".to_string() class="text-gray-400 animate-spin".to_string() />
                            <span class="text-xs text-gray-500 dark:text-gray-400">{"Loading thumbnail..."}</span>
                        </div>
                    </div>
                }.into_any()
            }
            ThumbnailState::Loaded => {
                // Log thumbnail rendering without exposing base64 data
                let current_data_url = data_url.get();
                web_sys::console::log_1(&format!("🖼️ Rendering thumbnail (data URL length: {} chars)", current_data_url.len()).into());

                view! {
                    <img
                        src=current_data_url
                        alt=alt.clone()
                        class=format!("{} object-cover", base_class)
                        loading="lazy"
                    />
                }.into_any()
            }
            ThumbnailState::Error => {
                view! {
                    <div class=format!("{} bg-gray-200 dark:bg-gray-700 flex items-center justify-center", base_class)>
                        <div class="flex flex-col items-center space-y-2">
                            <ImagePlaceholderIcon size="8".to_string() class="text-gray-400".to_string() />
                            <span class="text-xs text-gray-500 dark:text-gray-400">{"Failed to load thumbnail"}</span>
                        </div>
                    </div>
                }.into_any()
            }
            ThumbnailState::Empty => {
                view! {
                    <div class=format!("{} bg-gray-200 dark:bg-gray-700 flex items-center justify-center", base_class)>
                        <div class="flex flex-col items-center space-y-2">
                            <ImagePlaceholderIcon size="8".to_string() class="text-gray-400".to_string() />
                            <span class="text-xs text-gray-500 dark:text-gray-400">{"No thumbnail available"}</span>
                        </div>
                    </div>
                }.into_any()
            }
        }}
    }
}
