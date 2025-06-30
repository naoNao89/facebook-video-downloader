use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::console;
use crate::services::tauri_api;
use crate::components::icons::{ClipboardIcon, CheckIcon};

#[derive(Properties, PartialEq)]
pub struct CopyButtonProps {
    /// The text to copy to clipboard
    pub text: String,
    /// Optional custom class for styling
    #[prop_or_default]
    pub class: String,
    /// Size of the icon (default: "4")
    #[prop_or("4".to_string())]
    pub size: String,
    /// Optional tooltip text (default: "Copy to clipboard")
    #[prop_or("Copy to clipboard".to_string())]
    pub tooltip: String,
}

#[derive(Clone, PartialEq)]
enum CopyState {
    Idle,
    Copying,
    Success,
    Error(String),
}

#[function_component(CopyButton)]
pub fn copy_button(props: &CopyButtonProps) -> Html {
    let copy_state = use_state(|| CopyState::Idle);
    let timeout_handle = use_state(|| None::<i32>);

    let on_copy = {
        let text = props.text.clone();
        let copy_state = copy_state.clone();
        let timeout_handle = timeout_handle.clone();
        
        Callback::from(move |_: MouseEvent| {
            let text = text.clone();
            let copy_state = copy_state.clone();
            let timeout_handle = timeout_handle.clone();
            
            // Clear any existing timeout
            if let Some(handle) = *timeout_handle {
                web_sys::window()
                    .unwrap()
                    .clear_timeout_with_handle(handle);
            }
            
            copy_state.set(CopyState::Copying);
            
            spawn_local(async move {
                match tauri_api::write_clipboard(text.clone()).await {
                    Ok(_) => {
                        console::log_1(&format!("✅ Copied to clipboard: {}", text).into());
                        copy_state.set(CopyState::Success);
                        
                        // Reset to idle state after 2 seconds
                        let copy_state_reset = copy_state.clone();
                        let timeout_handle_set = timeout_handle.clone();
                        
                        let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                            copy_state_reset.set(CopyState::Idle);
                            timeout_handle_set.set(None);
                        }) as Box<dyn FnMut()>);
                        
                        let handle = web_sys::window()
                            .unwrap()
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                callback.as_ref().unchecked_ref(),
                                2000,
                            )
                            .unwrap();
                        
                        timeout_handle.set(Some(handle));
                        callback.forget(); // Prevent cleanup
                    }
                    Err(e) => {
                        console::log_1(&format!("❌ Failed to copy to clipboard: {}", e.error).into());
                        copy_state.set(CopyState::Error(e.error));
                        
                        // Reset to idle state after 3 seconds for errors
                        let copy_state_reset = copy_state.clone();
                        let timeout_handle_set = timeout_handle.clone();
                        
                        let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                            copy_state_reset.set(CopyState::Idle);
                            timeout_handle_set.set(None);
                        }) as Box<dyn FnMut()>);
                        
                        let handle = web_sys::window()
                            .unwrap()
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                callback.as_ref().unchecked_ref(),
                                3000,
                            )
                            .unwrap();
                        
                        timeout_handle.set(Some(handle));
                        callback.forget(); // Prevent cleanup
                    }
                }
            });
        })
    };

    let (button_class, _icon_class, tooltip_text, icon_element) = match &*copy_state {
        CopyState::Idle => (
            format!(
                "inline-flex items-center justify-center p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded transition-colors duration-200 hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-1 {}",
                props.class
            ),
            "".to_string(),
            props.tooltip.clone(),
            html! { <ClipboardIcon size={props.size.clone()} class="" /> }
        ),
        CopyState::Copying => (
            format!(
                "inline-flex items-center justify-center p-1 text-blue-500 dark:text-blue-400 rounded transition-colors duration-200 cursor-wait {}",
                props.class
            ),
            "animate-pulse".to_string(),
            "Copying...".to_string(),
            html! { <ClipboardIcon size={props.size.clone()} class="animate-pulse" /> }
        ),
        CopyState::Success => (
            format!(
                "inline-flex items-center justify-center p-1 text-green-500 dark:text-green-400 rounded transition-colors duration-200 {}",
                props.class
            ),
            "".to_string(),
            "Copied!".to_string(),
            html! { <CheckIcon size={props.size.clone()} class="" /> }
        ),
        CopyState::Error(_) => (
            format!(
                "inline-flex items-center justify-center p-1 text-red-500 dark:text-red-400 rounded transition-colors duration-200 {}",
                props.class
            ),
            "".to_string(),
            "Failed to copy".to_string(),
            html! { <ClipboardIcon size={props.size.clone()} class="" /> }
        ),
    };

    html! {
        <button
            class={button_class}
            onclick={on_copy}
            title={tooltip_text.clone()}
            aria-label={tooltip_text}
            disabled={matches!(*copy_state, CopyState::Copying)}
        >
            {icon_element}
        </button>
    }
}
