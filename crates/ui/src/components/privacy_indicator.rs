use yew::prelude::*;
use crate::components::{GlobeIcon, LockIcon, UsersIcon};

#[derive(Properties, PartialEq)]
pub struct PrivacyIndicatorProps {
    pub privacy_level: String,
    #[prop_or_default]
    pub class: String,
    #[prop_or("4".to_string())]
    pub size: String,
    #[prop_or(false)]
    pub show_label: bool,
}

/// Privacy indicator component that shows appropriate icon and styling based on privacy level
#[function_component(PrivacyIndicator)]
pub fn privacy_indicator(props: &PrivacyIndicatorProps) -> Html {
    let (icon, label, color_class, bg_class) = match props.privacy_level.to_lowercase().as_str() {
        "public" => (
            html! { <GlobeIcon size={props.size.clone()} class="text-green-600 dark:text-green-400" /> },
            "Public",
            "text-green-600 dark:text-green-400",
            "bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800"
        ),
        "private" => (
            html! { <LockIcon size={props.size.clone()} class="text-red-600 dark:text-red-400" /> },
            "Private",
            "text-red-600 dark:text-red-400",
            "bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800"
        ),
        "friends" => (
            html! { <UsersIcon size={props.size.clone()} class="text-blue-600 dark:text-blue-400" /> },
            "Friends Only",
            "text-blue-600 dark:text-blue-400",
            "bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800"
        ),
        _ => (
            html! { <div class="w-4 h-4 rounded-full bg-gray-400 dark:bg-gray-600"></div> },
            "Unknown",
            "text-gray-600 dark:text-gray-400",
            "bg-gray-50 dark:bg-gray-900/20 border-gray-200 dark:border-gray-800"
        ),
    };

    let container_class = if props.show_label {
        format!("inline-flex items-center space-x-2 px-2 py-1 rounded-md border text-xs font-medium {} {}", bg_class, props.class)
    } else {
        format!("inline-flex items-center {}", props.class)
    };

    html! {
        <div class={container_class} title={format!("Privacy: {}", label)}>
            {icon}
            if props.show_label {
                <span class={color_class}>{label}</span>
            }
        </div>
    }
}

/// Compact privacy indicator for use in video info cards
#[function_component(CompactPrivacyIndicator)]
pub fn compact_privacy_indicator(props: &PrivacyIndicatorProps) -> Html {
    let (icon, label, color_class) = match props.privacy_level.to_lowercase().as_str() {
        "public" => (
            html! { <GlobeIcon size="3" class="text-green-600 dark:text-green-400" /> },
            "Public video",
            "text-green-600 dark:text-green-400"
        ),
        "private" => (
            html! { <LockIcon size="3" class="text-red-600 dark:text-red-400" /> },
            "Private video",
            "text-red-600 dark:text-red-400"
        ),
        "friends" => (
            html! { <UsersIcon size="3" class="text-blue-600 dark:text-blue-400" /> },
            "Friends only",
            "text-blue-600 dark:text-blue-400"
        ),
        _ => (
            html! { <div class="w-3 h-3 rounded-full bg-gray-400 dark:bg-gray-600"></div> },
            "Privacy unknown",
            "text-gray-600 dark:text-gray-400"
        ),
    };

    html! {
        <div class={format!("inline-flex items-center space-x-1 {}", props.class)} title={label}>
            {icon}
            if props.show_label {
                <span class={format!("text-xs {}", color_class)}>{label}</span>
            }
        </div>
    }
}
