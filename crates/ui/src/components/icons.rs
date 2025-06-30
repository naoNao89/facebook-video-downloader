use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct IconProps {
    #[prop_or_default]
    pub class: String,
    #[prop_or("24".to_string())]
    pub size: String,
}

/// Clipboard icon component
#[function_component(ClipboardIcon)]
pub fn clipboard_icon(props: &IconProps) -> Html {
    let size = &props.size;
    let class = format!("w-{} h-{} {}", size, size, props.class);
    
    html! {
        <svg 
            class={class}
            fill="none" 
            stroke="currentColor" 
            viewBox="0 0 24 24" 
            xmlns="http://www.w3.org/2000/svg"
        >
            <path 
                stroke-linecap="round" 
                stroke-linejoin="round" 
                stroke-width="2" 
                d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
            />
        </svg>
    }
}

/// Paste icon component (clipboard with arrow)
#[function_component(PasteIcon)]
pub fn paste_icon(props: &IconProps) -> Html {
    let size = &props.size;
    let class = format!("w-{} h-{} {}", size, size, props.class);
    
    html! {
        <svg 
            class={class}
            fill="none" 
            stroke="currentColor" 
            viewBox="0 0 24 24" 
            xmlns="http://www.w3.org/2000/svg"
        >
            <path 
                stroke-linecap="round" 
                stroke-linejoin="round" 
                stroke-width="2" 
                d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"
            />
        </svg>
    }
}

/// Loading spinner icon
#[function_component(LoadingIcon)]
pub fn loading_icon(props: &IconProps) -> Html {
    let size = &props.size;
    let class = format!("w-{} h-{} animate-spin {}", size, size, props.class);
    
    html! {
        <svg 
            class={class}
            fill="none" 
            viewBox="0 0 24 24"
        >
            <circle 
                class="opacity-25" 
                cx="12" 
                cy="12" 
                r="10" 
                stroke="currentColor" 
                stroke-width="4"
            />
            <path 
                class="opacity-75" 
                fill="currentColor" 
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
        </svg>
    }
}

/// Image placeholder icon for missing thumbnails
#[function_component(ImagePlaceholderIcon)]
pub fn image_placeholder_icon(props: &IconProps) -> Html {
    let size = &props.size;
    let class = format!("w-{} h-{} {}", size, size, props.class);
    
    html! {
        <svg 
            class={class}
            fill="currentColor" 
            viewBox="0 0 24 24" 
            xmlns="http://www.w3.org/2000/svg"
        >
            <path 
                fill-rule="evenodd" 
                d="M4 3a2 2 0 00-2 2v14a2 2 0 002 2h16a2 2 0 002-2V5a2 2 0 00-2-2H4zm10 5a3 3 0 11-6 0 3 3 0 016 0zm-3-1a1 1 0 100 2 1 1 0 000-2zm-5 9v-2a2 2 0 012-2h8a2 2 0 012 2v2H6z" 
                clip-rule="evenodd"
            />
        </svg>
    }
}

/// Video play icon for thumbnails
#[function_component(PlayIcon)]
pub fn play_icon(props: &IconProps) -> Html {
    let size = &props.size;
    let class = format!("w-{} h-{} {}", size, size, props.class);

    html! {
        <svg
            class={class}
            fill="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M8 5v14l11-7z"
            />
        </svg>
    }
}

/// Check icon for success feedback
#[function_component(CheckIcon)]
pub fn check_icon(props: &IconProps) -> Html {
    let size = &props.size;
    let class = format!("w-{} h-{} {}", size, size, props.class);

    html! {
        <svg
            class={class}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M5 13l4 4L19 7"
            />
        </svg>
    }
}

/// Globe icon component (for public indicator)
#[function_component(GlobeIcon)]
pub fn globe_icon(props: &IconProps) -> Html {
    let size = &props.size;
    let class = format!("w-{} h-{} {}", size, size, props.class);

    html! {
        <svg
            class={class}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"
            />
        </svg>
    }
}

/// Lock icon component (for private indicator)
#[function_component(LockIcon)]
pub fn lock_icon(props: &IconProps) -> Html {
    let size = &props.size;
    let class = format!("w-{} h-{} {}", size, size, props.class);

    html! {
        <svg
            class={class}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
            />
        </svg>
    }
}

/// Users icon component (for friends-only indicator)
#[function_component(UsersIcon)]
pub fn users_icon(props: &IconProps) -> Html {
    let size = &props.size;
    let class = format!("w-{} h-{} {}", size, size, props.class);

    html! {
        <svg
            class={class}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a4 4 0 11-8 0 4 4 0 018 0z"
            />
        </svg>
    }
}
