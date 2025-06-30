use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::console;
use crate::services::tauri_api;
use crate::components::icons::{ClipboardIcon, CheckIcon};

#[derive(Clone, PartialEq, Copy)]
enum CopyState {
    Idle,
    Copying,
    Success,
    Error,
}

#[component]
pub fn CopyButton(
    /// The text to copy to clipboard
    text: String,
    /// Optional custom class for styling
    #[prop(optional)] class: Option<String>,
    /// Size of the icon (default: "4")
    #[prop(default = "4".to_string())] size: String,
    /// Optional tooltip text (default: "Copy to clipboard")
    #[prop(default = "Copy to clipboard".to_string())] tooltip: String,
) -> impl IntoView {
    let (copy_state, set_copy_state) = signal(CopyState::Idle);
    let (timeout_handle, set_timeout_handle) = signal(None::<i32>);

    let on_copy = move |_| {
        let text = text.clone();

        // Clear any existing timeout
        if let Some(handle) = timeout_handle.get() {
            web_sys::window()
                .unwrap()
                .clear_timeout_with_handle(handle);
        }

        set_copy_state.set(CopyState::Copying);

        spawn_local(async move {
            match tauri_api::write_clipboard(text.clone()).await {
                Ok(_) => {
                    console::log_1(&format!("✅ Copied to clipboard: {}", text).into());
                    set_copy_state.set(CopyState::Success);

                    // Reset to idle state after 2 seconds
                    let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                        set_copy_state.set(CopyState::Idle);
                        set_timeout_handle.set(None);
                    }) as Box<dyn FnMut()>);

                    let handle = web_sys::window()
                        .unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            callback.as_ref().unchecked_ref(),
                            2000,
                        )
                        .unwrap();

                    set_timeout_handle.set(Some(handle));
                    callback.forget(); // Prevent cleanup
                }
                Err(e) => {
                    console::log_1(&format!("❌ Failed to copy to clipboard: {}", e.error).into());
                    set_copy_state.set(CopyState::Error);

                    // Reset to idle state after 3 seconds for errors
                    let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                        set_copy_state.set(CopyState::Idle);
                        set_timeout_handle.set(None);
                    }) as Box<dyn FnMut()>);

                    let handle = web_sys::window()
                        .unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            callback.as_ref().unchecked_ref(),
                            3000,
                        )
                        .unwrap();

                    set_timeout_handle.set(Some(handle));
                    callback.forget(); // Prevent cleanup
                }
            }
        });
    };

    let button_class = move || {
        let base_class = class.as_deref().unwrap_or("");
        match copy_state.get() {
            CopyState::Idle => format!(
                "inline-flex items-center justify-center p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded transition-colors duration-200 hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-1 {}",
                base_class
            ),
            CopyState::Copying => format!(
                "inline-flex items-center justify-center p-1 text-blue-500 dark:text-blue-400 rounded transition-colors duration-200 cursor-wait {}",
                base_class
            ),
            CopyState::Success => format!(
                "inline-flex items-center justify-center p-1 text-green-500 dark:text-green-400 rounded transition-colors duration-200 {}",
                base_class
            ),
            CopyState::Error => format!(
                "inline-flex items-center justify-center p-1 text-red-500 dark:text-red-400 rounded transition-colors duration-200 {}",
                base_class
            ),
        }
    };

    let tooltip_clone = tooltip.clone();
    let tooltip_text = move || {
        match copy_state.get() {
            CopyState::Idle => tooltip_clone.clone(),
            CopyState::Copying => "Copying...".to_string(),
            CopyState::Success => "Copied!".to_string(),
            CopyState::Error => "Failed to copy".to_string(),
        }
    };

    let tooltip_text_aria = move || {
        match copy_state.get() {
            CopyState::Idle => tooltip.clone(),
            CopyState::Copying => "Copying...".to_string(),
            CopyState::Success => "Copied!".to_string(),
            CopyState::Error => "Failed to copy".to_string(),
        }
    };

    let icon_element = move || {
        match copy_state.get() {
            CopyState::Idle => view! { <ClipboardIcon size=size.clone() class="".to_string()/> }.into_any(),
            CopyState::Copying => view! { <ClipboardIcon size=size.clone() class="animate-pulse".to_string()/> }.into_any(),
            CopyState::Success => view! { <CheckIcon size=size.clone() class="".to_string()/> }.into_any(),
            CopyState::Error => view! { <ClipboardIcon size=size.clone() class="".to_string()/> }.into_any(),
        }
    };

    view! {
        <button
            class=button_class
            on:click=on_copy
            title=tooltip_text
            aria-label=tooltip_text_aria
            disabled=move || matches!(copy_state.get(), CopyState::Copying)
        >
            {icon_element}
        </button>
    }
}
