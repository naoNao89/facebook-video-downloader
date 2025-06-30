use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::tauri_api;
use crate::components::icons::{ImagePlaceholderIcon, LoadingIcon};

#[derive(Properties, PartialEq)]
pub struct ThumbnailProps {
    pub url: String,
    pub alt: String,
    #[prop_or_default]
    pub class: String,
}

#[derive(Clone, PartialEq)]
enum ThumbnailState {
    Loading,
    Loaded(String), // Base64 data URL
    Error,
    Empty,
}

#[function_component(Thumbnail)]
pub fn thumbnail(props: &ThumbnailProps) -> Html {
    let thumbnail_state = use_state(|| {
        if props.url.is_empty() || props.url.contains("placeholder") {
            ThumbnailState::Empty
        } else {
            ThumbnailState::Loading
        }
    });

    let url = props.url.clone();
    let thumbnail_state_clone = thumbnail_state.clone();

    // Effect to fetch thumbnail when URL changes
    use_effect_with(url.clone(), move |url| {
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
            thumbnail_state_clone.set(ThumbnailState::Empty);
            return;
        }

        web_sys::console::log_1(&"🖼️ Setting state to Loading".into());
        thumbnail_state_clone.set(ThumbnailState::Loading);

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

        let state = thumbnail_state_clone.clone();

        // Check if the URL is already a data URL (from fallback SVG)
        if decoded_url.starts_with("data:") {
            web_sys::console::log_1(&"🖼️ [Thumbnail] Using data URL for thumbnail display".into());

            // Validate data URL format
            if decoded_url.starts_with("data:image/") {
                web_sys::console::log_1(&"📊 [Thumbnail] Thumbnail status: Valid data URL (image)".into());
                state.set(ThumbnailState::Loaded(decoded_url));
            } else if decoded_url.starts_with("data:") {
                web_sys::console::log_1(&"📊 [Thumbnail] Thumbnail status: Valid data URL (non-image)".into());
                state.set(ThumbnailState::Loaded(decoded_url));
            } else {
                web_sys::console::error_1(&"❌ Invalid data URL format - setting to Error".into());
                web_sys::console::log_1(&"📊 [Thumbnail] Thumbnail status: Error (invalid data URL)".into());
                state.set(ThumbnailState::Error);
            }
            return;
        }

        // For HTTP URLs, fetch through Tauri API
        spawn_local(async move {
            web_sys::console::log_1(&"🚀 Fetching thumbnail via Tauri API...".into());
            web_sys::console::log_1(&format!("🔍 HTTP URL: {}", decoded_url).into());

            match tauri_api::fetch_thumbnail_image(decoded_url.clone()).await {
                Ok(data_url) => {
                    web_sys::console::log_1(&format!("✅ Thumbnail fetched successfully! Data URL length: {}", data_url.len()).into());

                    // Test if the data URL is valid
                    if data_url.starts_with("data:image/") {
                        web_sys::console::log_1(&"✅ Data URL format is valid".into());
                        web_sys::console::log_1(&"🎯 Setting thumbnail state to Loaded".into());
                        state.set(ThumbnailState::Loaded(data_url));
                    } else if data_url.starts_with("data:") {
                        web_sys::console::log_1(&"✅ Data URL format detected (non-image)".into());
                        web_sys::console::log_1(&"🎯 Setting thumbnail state to Loaded (placeholder)".into());
                        state.set(ThumbnailState::Loaded(data_url));
                    } else {
                        web_sys::console::error_1(&"❌ Invalid data URL format received".into());
                        web_sys::console::log_1(&"🎯 Setting thumbnail state to Error due to invalid format".into());
                        state.set(ThumbnailState::Error);
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("❌ Failed to fetch thumbnail from URL: {}", decoded_url).into());
                    web_sys::console::error_1(&format!("❌ Error details: {:?}", e).into());
                    web_sys::console::log_1(&"🎯 Setting thumbnail state to Error due to fetch failure".into());
                    state.set(ThumbnailState::Error);
                }
            }
        });
    });

    let base_class = format!("w-full h-full {}", props.class);

    match &*thumbnail_state {
        ThumbnailState::Loading => {
            html! {
                <div class={format!("{} bg-gray-200 dark:bg-gray-700 flex items-center justify-center", base_class)}>
                    <div class="flex flex-col items-center space-y-2">
                        <LoadingIcon size="8" class="text-gray-400 animate-spin" />
                        <span class="text-xs text-gray-500 dark:text-gray-400">{"Loading thumbnail..."}</span>
                    </div>
                </div>
            }
        }
        ThumbnailState::Loaded(data_url) => {
            // Log thumbnail rendering without exposing base64 data
            web_sys::console::log_1(&format!("🖼️ Rendering thumbnail (data URL length: {} chars)", data_url.len()).into());

            html! {
                <img
                    src={data_url.clone()}
                    alt={props.alt.clone()}
                    class={format!("{} object-cover", base_class)}
                    loading="lazy"
                />
            }
        }
        ThumbnailState::Error => {
            html! {
                <div class={format!("{} bg-gray-200 dark:bg-gray-700 flex items-center justify-center", base_class)}>
                    <div class="flex flex-col items-center space-y-2">
                        <ImagePlaceholderIcon size="8" class="text-gray-400" />
                        <span class="text-xs text-gray-500 dark:text-gray-400">{"Failed to load thumbnail"}</span>
                    </div>
                </div>
            }
        }
        ThumbnailState::Empty => {
            html! {
                <div class={format!("{} bg-gray-200 dark:bg-gray-700 flex items-center justify-center", base_class)}>
                    <div class="flex flex-col items-center space-y-2">
                        <ImagePlaceholderIcon size="8" class="text-gray-400" />
                        <span class="text-xs text-gray-500 dark:text-gray-400">{"No thumbnail available"}</span>
                    </div>
                </div>
            }
        }
    }
}
