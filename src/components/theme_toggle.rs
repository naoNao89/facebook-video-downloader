use leptos::*;
use crate::services::theme::{use_theme, Theme};

/// Theme toggle component that provides a button to switch between light and dark themes
#[component]
pub fn ThemeToggle() -> impl IntoView {
    let theme_context = use_theme();
    let current_theme = move || theme_context.theme.get();

    // Determine the next theme and appropriate icon
    let theme_info = move || {
        match current_theme() {
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
        }
    };

    let onclick = move |_| {
        let (next_theme, _, _) = theme_info();
        theme_context.set_theme.set(next_theme);
    };

    view! {
        <button
            on:click=onclick
            class="relative p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-facebook-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800"
            title=move || theme_info().2
            aria-label=move || theme_info().2
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
                    d=move || theme_info().1
                />
            </svg>

            // Add a subtle animation indicator
            <span class="absolute inset-0 rounded-lg bg-gradient-to-r from-facebook-500/0 via-facebook-500/10 to-facebook-500/0 opacity-0 hover:opacity-100 transition-opacity duration-300"></span>
        </button>
    }
}

/// Enhanced theme toggle with switch-style UI
#[component]
pub fn ThemeToggleSwitch(
    #[prop(optional)] class: Option<String>,
    #[prop(default = false)] show_label: bool,
) -> impl IntoView {
    let theme_context = use_theme();
    let is_dark = move || matches!(theme_context.theme.get(), Theme::Dark);

    let onclick = move |_| {
        let current_theme = theme_context.theme.get();
        let new_theme = match current_theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
            Theme::System => Theme::Dark,
        };
        theme_context.set_theme.set(new_theme);
    };

    let switch_classes = move || {
        let mut classes = vec![
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
        ];

        if is_dark() {
            classes.push("bg-facebook-600");
        } else {
            classes.push("bg-gray-200");
        }

        if let Some(ref additional_class) = class {
            classes.push(additional_class);
        }

        classes.join(" ")
    };

    let toggle_classes = move || {
        let mut classes = vec![
            "inline-block",
            "h-4",
            "w-4",
            "transform",
            "rounded-full",
            "bg-white",
            "transition-transform",
            "duration-200",
            "shadow-lg",
        ];

        if is_dark() {
            classes.push("translate-x-6");
        } else {
            classes.push("translate-x-1");
        }

        classes.join(" ")
    };

    view! {
        <div class="flex items-center space-x-3">
            {move || if show_label {
                view! {
                    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                        "Theme"
                    </span>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
            <button
                on:click=onclick
                class=switch_classes
                role="switch"
                aria-checked=move || is_dark().to_string()
                aria-label="Toggle theme"
            >
                <span class=toggle_classes />
            </button>
            {move || if show_label {
                view! {
                    <span class="text-xs text-gray-500 dark:text-gray-400">
                        {move || if is_dark() { "Dark" } else { "Light" }}
                    </span>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}
