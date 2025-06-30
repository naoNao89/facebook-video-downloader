use leptos::*;
use gloo::storage::{LocalStorage, Storage as GlooStorage};
use wasm_bindgen::prelude::*;

const THEME_STORAGE_KEY: &str = "facebook-video-downloader-theme";

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn to_string(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "dark" => Theme::Dark,
            "system" => Theme::System,
            _ => Theme::Light,
        }
    }

    /// Get the effective theme (resolves System to Light/Dark based on system preference)
    pub fn effective_theme(&self) -> Theme {
        match self {
            Theme::System => {
                if is_system_dark_mode() {
                    Theme::Dark
                } else {
                    Theme::Light
                }
            }
            theme => *theme,
        }
    }
}

// JavaScript interop functions
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = "applyTheme")]
    fn apply_theme_js(theme: &str);

    #[wasm_bindgen(js_namespace = window, js_name = "isSystemDarkMode")]
    fn is_system_dark_mode_js() -> bool;
}

/// Check if the system prefers dark mode
fn is_system_dark_mode() -> bool {
    // Use JavaScript function if available, otherwise default to false
    match std::panic::catch_unwind(|| is_system_dark_mode_js()) {
        Ok(result) => result,
        Err(_) => false,
    }
}

/// Apply theme to the document element
fn apply_theme_to_document(theme: &Theme) {
    let effective_theme = theme.effective_theme();
    let theme_str = match effective_theme {
        Theme::Dark => "dark",
        Theme::Light => "light",
        Theme::System => "light", // This shouldn't happen as effective_theme resolves System
    };

    // Use JavaScript function if available
    if let Err(_) = std::panic::catch_unwind(|| apply_theme_js(theme_str)) {
        log::warn!("Failed to apply theme via JavaScript, theme functions may not be available");
    }

    log::info!("Applied theme: {:?} (effective: {:?})", theme, effective_theme);
}

/// Save theme preference to localStorage
fn save_theme_preference(theme: &Theme) {
    if let Err(e) = LocalStorage::set(THEME_STORAGE_KEY, theme.to_string()) {
        log::error!("Failed to save theme preference: {:?}", e);
    } else {
        log::info!("Saved theme preference: {:?}", theme);
    }
}

/// Load theme preference from localStorage
fn load_theme_preference() -> Option<Theme> {
    match LocalStorage::get::<String>(THEME_STORAGE_KEY) {
        Ok(theme_str) => {
            let theme = Theme::from_string(&theme_str);
            log::info!("Loaded theme preference: {:?}", theme);
            Some(theme)
        }
        Err(_) => {
            log::info!("No theme preference found, using system default");
            None
        }
    }
}

#[derive(Clone, Copy)]
pub struct ThemeContext {
    pub theme: ReadSignal<Theme>,
    pub set_theme: WriteSignal<Theme>,
}

#[component]
pub fn ThemeProvider(children: Children) -> impl IntoView {
    // Initialize theme from localStorage or default to System
    let (theme, set_theme) = create_signal(
        load_theme_preference().unwrap_or(Theme::System)
    );

    // Apply theme on mount and when theme changes
    create_effect(move |_| {
        let current_theme = theme.get();
        apply_theme_to_document(&current_theme);
    });

    // Create a custom setter that includes side effects
    let set_theme_with_effects = move |new_theme: Theme| {
        log::info!("Setting theme to: {:?}", new_theme);

        // Update state
        set_theme.set(new_theme);

        // Save to localStorage
        save_theme_preference(&new_theme);

        // Apply to document
        apply_theme_to_document(&new_theme);
    };

    // TODO: Listen for system theme changes when using System theme
    // This will be implemented when web-sys properly supports matchMedia

    let context = ThemeContext {
        theme,
        set_theme,
    };

    provide_context(context);

    view! {
        {children()}
    }
}

pub fn use_theme() -> ThemeContext {
    expect_context::<ThemeContext>()
}
