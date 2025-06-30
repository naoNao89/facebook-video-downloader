use leptos::prelude::*;
use crate::components::{GlobeIcon, LockIcon, UsersIcon};

/// Privacy indicator component that shows appropriate icon and styling based on privacy level
#[component]
pub fn PrivacyIndicator(
    privacy_level: String,
    #[prop(default = String::new())] class: String,
    #[prop(default = "4".to_string())] size: String,
    #[prop(default = false)] show_label: bool,
) -> impl IntoView {
    let (icon, label, color_class, bg_class) = match privacy_level.to_lowercase().as_str() {
        "public" => (
            view! { <GlobeIcon size=size.clone() class="text-green-600 dark:text-green-400".to_string() /> }.into_any(),
            "Public",
            "text-green-600 dark:text-green-400",
            "bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800"
        ),
        "private" => (
            view! { <LockIcon size=size.clone() class="text-red-600 dark:text-red-400".to_string() /> }.into_any(),
            "Private",
            "text-red-600 dark:text-red-400",
            "bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800"
        ),
        "friends" => (
            view! { <UsersIcon size=size.clone() class="text-blue-600 dark:text-blue-400".to_string() /> }.into_any(),
            "Friends Only",
            "text-blue-600 dark:text-blue-400",
            "bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800"
        ),
        _ => (
            view! { <div class="w-4 h-4 rounded-full bg-gray-400 dark:bg-gray-600"></div> }.into_any(),
            "Unknown",
            "text-gray-600 dark:text-gray-400",
            "bg-gray-50 dark:bg-gray-900/20 border-gray-200 dark:border-gray-800"
        ),
    };

    let container_class = if show_label {
        format!("inline-flex items-center space-x-2 px-2 py-1 rounded-md border text-xs font-medium {} {}", bg_class, class)
    } else {
        format!("inline-flex items-center {}", class)
    };

    view! {
        <div class=container_class title=format!("Privacy: {}", label)>
            {icon}
            <Show when=move || show_label>
                <span class=color_class>{label}</span>
            </Show>
        </div>
    }
}

/// Compact privacy indicator for use in video info cards
#[component]
pub fn CompactPrivacyIndicator(
    privacy_level: String,
    #[prop(default = String::new())] class: String,
    #[prop(default = false)] show_label: bool,
) -> impl IntoView {
    let (icon, label, color_class) = match privacy_level.to_lowercase().as_str() {
        "public" => (
            view! { <GlobeIcon size="3".to_string() class="text-green-600 dark:text-green-400".to_string() /> }.into_any(),
            "Public video",
            "text-green-600 dark:text-green-400"
        ),
        "private" => (
            view! { <LockIcon size="3".to_string() class="text-red-600 dark:text-red-400".to_string() /> }.into_any(),
            "Private video",
            "text-red-600 dark:text-red-400"
        ),
        "friends" => (
            view! { <UsersIcon size="3".to_string() class="text-blue-600 dark:text-blue-400".to_string() /> }.into_any(),
            "Friends only",
            "text-blue-600 dark:text-blue-400"
        ),
        _ => (
            view! { <div class="w-3 h-3 rounded-full bg-gray-400 dark:bg-gray-600"></div> }.into_any(),
            "Privacy unknown",
            "text-gray-600 dark:text-gray-400"
        ),
    };

    view! {
        <div class=format!("inline-flex items-center space-x-1 {}", class) title=label>
            {icon}
            <Show when=move || show_label>
                <span class=format!("text-xs {}", color_class)>{label}</span>
            </Show>
        </div>
    }
}
