use yew::prelude::*;
use crate::services::theme::{use_theme, Theme};

/// Theme toggle component that provides a button to switch between light and dark themes
#[function_component(ThemeToggle)]
pub fn theme_toggle() -> Html {
    let theme_context = use_theme();
    let current_theme = theme_context.theme.clone();
    
    // Determine the next theme and appropriate icon
    let (next_theme, icon_path, tooltip) = match current_theme {
        Theme::Light => (
            Theme::Dark,
            "M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z",
            "Switch to dark mode"
        ),
        Theme::Dark => (
            Theme::Light,
            "M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z",
            "Switch to light mode"
        ),
        Theme::System => (
            Theme::Light,
            "M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z",
            "Switch to light mode"
        ),
    };

    let onclick = {
        let set_theme = theme_context.set_theme.clone();
        Callback::from(move |_: MouseEvent| {
            set_theme.emit(next_theme.clone());
        })
    };

    html! {
        <button
            {onclick}
            class="relative p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-facebook-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800"
            title={tooltip}
            aria-label={tooltip}
        >
            <svg 
                class="w-5 h-5 transition-transform duration-200 hover:scale-110" 
                fill="none" 
                stroke="currentColor" 
                viewBox="0 0 24 24"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path 
                    stroke-linecap="round" 
                    stroke-linejoin="round" 
                    stroke-width="2" 
                    d={icon_path}
                />
            </svg>
            
            // Add a subtle animation indicator
            <span class="absolute inset-0 rounded-lg bg-gradient-to-r from-facebook-500/0 via-facebook-500/10 to-facebook-500/0 opacity-0 hover:opacity-100 transition-opacity duration-300"></span>
        </button>
    }
}

/// Enhanced theme toggle with switch-style UI
#[derive(Properties, PartialEq)]
pub struct ThemeToggleSwitchProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or(false)]
    pub show_label: bool,
}

#[function_component(ThemeToggleSwitch)]
pub fn theme_toggle_switch(props: &ThemeToggleSwitchProps) -> Html {
    let theme_context = use_theme();
    let is_dark = matches!(theme_context.theme, Theme::Dark);
    
    let onclick = {
        let set_theme = theme_context.set_theme.clone();
        let current_theme = theme_context.theme.clone();
        Callback::from(move |_: MouseEvent| {
            let new_theme = match current_theme {
                Theme::Light => Theme::Dark,
                Theme::Dark => Theme::Light,
                Theme::System => Theme::Dark,
            };
            set_theme.emit(new_theme);
        })
    };

    let switch_classes = classes!(
        "relative",
        "inline-flex",
        "h-6",
        "w-11",
        "items-center",
        "rounded-full",
        "transition-colors",
        "duration-200",
        "focus:outline-none",
        "focus:ring-2",
        "focus:ring-facebook-500",
        "focus:ring-offset-2",
        "dark:focus:ring-offset-gray-800",
        "cursor-pointer",
        if is_dark { "bg-facebook-600" } else { "bg-gray-200" },
        props.class.clone()
    );

    let toggle_classes = classes!(
        "inline-block",
        "h-4",
        "w-4",
        "transform",
        "rounded-full",
        "bg-white",
        "transition-transform",
        "duration-200",
        "shadow-lg",
        if is_dark { "translate-x-6" } else { "translate-x-1" }
    );

    html! {
        <div class="flex items-center space-x-3">
            if props.show_label {
                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                    {"Theme"}
                </span>
            }
            <button
                {onclick}
                class={switch_classes}
                role="switch"
                aria-checked={is_dark.to_string()}
                aria-label="Toggle theme"
            >
                <span class={toggle_classes} />
            </button>
            if props.show_label {
                <span class="text-xs text-gray-500 dark:text-gray-400">
                    {if is_dark { "Dark" } else { "Light" }}
                </span>
            }
        </div>
    }
}
