use yew::prelude::*;
use gloo::storage::{LocalStorage, Storage as GlooStorage};
use wasm_bindgen::prelude::*;

const THEME_STORAGE_KEY: &str = "facebook-video-downloader-theme";

#[derive(Clone, PartialEq, Debug)]
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
            theme => theme.clone(),
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

#[derive(Clone)]
pub struct ThemeContext {
    pub theme: Theme,
    pub set_theme: Callback<Theme>,
}

impl PartialEq for ThemeContext {
    fn eq(&self, other: &Self) -> bool {
        self.theme == other.theme
    }
}

#[derive(Properties, PartialEq)]
pub struct ThemeProviderProps {
    pub children: Children,
}

#[function_component(ThemeProvider)]
pub fn theme_provider(props: &ThemeProviderProps) -> Html {
    // Initialize theme from localStorage or default to System
    let theme = use_state(|| {
        load_theme_preference().unwrap_or(Theme::System)
    });

    let set_theme = {
        let theme = theme.clone();
        Callback::from(move |new_theme: Theme| {
            log::info!("Setting theme to: {:?}", new_theme);

            // Update state
            theme.set(new_theme.clone());

            // Save to localStorage
            save_theme_preference(&new_theme);

            // Apply to document
            apply_theme_to_document(&new_theme);
        })
    };

    // Apply theme on mount and when theme changes
    {
        let current_theme = (*theme).clone();
        use_effect_with(current_theme.clone(), move |theme| {
            apply_theme_to_document(theme);
            || ()
        });
    }

    // TODO: Listen for system theme changes when using System theme
    // This will be implemented when web-sys properly supports matchMedia

    let context = ThemeContext {
        theme: (*theme).clone(),
        set_theme,
    };

    html! {
        <ContextProvider<ThemeContext> context={context}>
            {props.children.clone()}
        </ContextProvider<ThemeContext>>
    }
}

#[hook]
pub fn use_theme() -> ThemeContext {
    use_context::<ThemeContext>().expect("Theme context not found")
}
