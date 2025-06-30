mod app;
mod components;
mod pages;
mod services;
mod hooks;
mod utils;

use app::App;
use leptos::*;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

// This is the entry point for the web app
#[wasm_bindgen(start)]
pub fn run_app() {
    // Initialize console logging for development
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);

    // Log application startup
    log::info!("Facebook Video Downloader starting...");
    log::info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Mount the Leptos application
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
