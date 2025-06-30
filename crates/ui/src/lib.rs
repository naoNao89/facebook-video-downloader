mod app;
mod components;
mod pages;
mod services;
mod hooks;
mod utils;

use app::App;
use wasm_bindgen::prelude::*;

// This is the entry point for the web app
#[wasm_bindgen(start)]
pub fn run_app() {
    // Initialize console logging for development
    wasm_logger::init(wasm_logger::Config::default());
    
    // Log application startup
    log::info!("Facebook Video Downloader starting...");
    log::info!("Version: {}", env!("CARGO_PKG_VERSION"));
    
    // Mount the Yew application
    yew::Renderer::<App>::new().render();
}
