use leptos::prelude::*;
use web_sys::MouseEvent;
use wasm_bindgen::JsCast;

#[derive(Clone, PartialEq)]
pub enum ModalSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl Default for ModalSize {
    fn default() -> Self {
        Self::Medium
    }
}

impl ModalSize {
    fn to_class(&self) -> &'static str {
        match self {
            ModalSize::Small => "max-w-md",
            ModalSize::Medium => "max-w-lg",
            ModalSize::Large => "max-w-2xl",
            ModalSize::ExtraLarge => "max-w-4xl",
        }
    }
}

#[component]
pub fn Modal(
    is_open: ReadSignal<bool>,
    on_close: impl Fn() + 'static,
    title: String,
    children: Children,
    #[prop(optional)] size: ModalSize,
    #[prop(default = true)] closable: bool,
    #[prop(default = true)] show_close_button: bool,
) -> impl IntoView {
    let on_backdrop_click = move |e: MouseEvent| {
        if closable {
            // Only close if clicking the backdrop, not the modal content
            if let Some(target) = e.target() {
                if let Some(element) = target.dyn_ref::<web_sys::Element>() {
                    if element.class_name().contains("modal-backdrop") {
                        on_close();
                    }
                }
            }
        }
    };

    let on_close_button_click = move |_: MouseEvent| {
        on_close();
    };

    view! {
        <Show when=move || is_open.get()>
            <div class="fixed inset-0 z-50 overflow-y-auto">
                <div class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
                    // Background overlay
                    <div
                        class="modal-backdrop fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
                        on:click=on_backdrop_click
                    ></div>

                    // This element is to trick the browser into centering the modal contents
                    <span class="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">"​"</span>

                    // Modal panel
                    <div class=format!(
                        "inline-block align-bottom bg-white dark:bg-gray-800 rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle {} w-full",
                        size.to_class()
                    )>
                        // Header
                        <div class="bg-white dark:bg-gray-800 px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
                            <div class="flex items-center justify-between mb-4">
                                <h3 class="text-lg leading-6 font-medium text-gray-900 dark:text-white" id="modal-title">
                                    {title.clone()}
                                </h3>
                                <Show when=move || show_close_button>
                                    <button
                                        type="button"
                                        class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-facebook-500"
                                        on:click=on_close_button_click
                                    >
                                        <span class="sr-only">"Close"</span>
                                        <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                        </svg>
                                    </button>
                                </Show>
                            </div>

                            // Content
                            <div class="mt-2">
                                {children()}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}



#[component]
pub fn ConfirmModal(
    is_open: ReadSignal<bool>,
    on_close: impl Fn() + 'static + Clone,
    on_confirm: impl Fn() + 'static,
    title: String,
    message: String,
    #[prop(default = "Confirm".to_string())] confirm_text: String,
    #[prop(default = "Cancel".to_string())] cancel_text: String,
    #[prop(default = false)] danger: bool,
) -> impl IntoView {
    let on_close_clone = on_close.clone();
    let on_confirm_click = move |_| {
        on_confirm();
        on_close_clone();
    };

    let on_cancel_click = move |_| {
        on_close();
    };

    let confirm_button_class = if danger {
        "inline-flex justify-center w-full rounded-md border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:ml-3 sm:w-auto sm:text-sm"
    } else {
        "inline-flex justify-center w-full rounded-md border border-transparent shadow-sm px-4 py-2 bg-facebook-600 text-base font-medium text-white hover:bg-facebook-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-facebook-500 sm:ml-3 sm:w-auto sm:text-sm"
    };

    view! {
        <Modal
            is_open=is_open
            on_close=on_close
            title=title
            size=ModalSize::Small
            closable=true
            show_close_button=false
        >
            <div class="mt-2">
                <p class="text-sm text-gray-500 dark:text-gray-400">
                    {message.clone()}
                </p>
            </div>

            <div class="mt-5 sm:mt-6 sm:grid sm:grid-cols-2 sm:gap-3 sm:grid-flow-row-dense">
                <button
                    type="button"
                    class=confirm_button_class
                    on:click=on_confirm_click
                >
                    {confirm_text}
                </button>
                <button
                    type="button"
                    class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 dark:border-gray-600 shadow-sm px-4 py-2 bg-white dark:bg-gray-700 text-base font-medium text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-facebook-500 sm:mt-0 sm:col-start-1 sm:text-sm"
                    on:click=on_cancel_click
                >
                    {cancel_text}
                </button>
            </div>
        </Modal>
    }
}
