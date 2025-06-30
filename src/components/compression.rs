use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use crate::services::tauri_api;

/// Compression levels including original (no compression)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompressionLevel {
    Original,  // 100% - no compression
    High,      // 90% quality
    Medium,    // 50% quality
    Low,       // 30% quality
    Minimal,   // 10% quality
}

impl CompressionLevel {
    pub fn percentage(&self) -> u8 {
        match self {
            CompressionLevel::Original => 100,
            CompressionLevel::High => 90,
            CompressionLevel::Medium => 50,
            CompressionLevel::Low => 30,
            CompressionLevel::Minimal => 10,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CompressionLevel::Original => "Original quality, full file size",
            CompressionLevel::High => "High quality, smaller file",
            CompressionLevel::Medium => "Balanced quality/size",
            CompressionLevel::Low => "Lower quality, smaller file",
            CompressionLevel::Minimal => "Lowest quality, minimal file size",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CompressionLevel::Original => "100% Original",
            CompressionLevel::High => "90% Quality",
            CompressionLevel::Medium => "50% Quality",
            CompressionLevel::Low => "30% Quality",
            CompressionLevel::Minimal => "10% Quality",
        }
    }

    /// Get color classes for visual compression level indication
    pub fn color_classes(&self) -> &'static str {
        match self {
            CompressionLevel::Original => "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-200 dark:border-green-800",
            CompressionLevel::High => "bg-blue-100 text-blue-800 border-blue-200 dark:bg-blue-900/20 dark:text-blue-200 dark:border-blue-800",
            CompressionLevel::Medium => "bg-yellow-100 text-yellow-800 border-yellow-200 dark:bg-yellow-900/20 dark:text-yellow-200 dark:border-yellow-800",
            CompressionLevel::Low => "bg-orange-100 text-orange-800 border-orange-200 dark:bg-orange-900/20 dark:text-orange-200 dark:border-orange-800",
            CompressionLevel::Minimal => "bg-red-100 text-red-800 border-red-200 dark:bg-red-900/20 dark:text-red-200 dark:border-red-800",
        }
    }

    /// Get hover color classes for interactive elements
    pub fn hover_color_classes(&self) -> &'static str {
        match self {
            CompressionLevel::Original => "hover:bg-green-200 dark:hover:bg-green-800/30",
            CompressionLevel::High => "hover:bg-blue-200 dark:hover:bg-blue-800/30",
            CompressionLevel::Medium => "hover:bg-yellow-200 dark:hover:bg-yellow-800/30",
            CompressionLevel::Low => "hover:bg-orange-200 dark:hover:bg-orange-800/30",
            CompressionLevel::Minimal => "hover:bg-red-200 dark:hover:bg-red-800/30",
        }
    }

    /// Get compression intensity indicator (for accessibility)
    pub fn intensity_indicator(&self) -> &'static str {
        match self {
            CompressionLevel::Original => "●", // Full circle
            CompressionLevel::High => "◐", // Half circle
            CompressionLevel::Medium => "◑", // Half circle alt
            CompressionLevel::Low => "◒", // Quarter circle
            CompressionLevel::Minimal => "○", // Empty circle
        }
    }

    pub fn is_compressed(&self) -> bool {
        !matches!(self, CompressionLevel::Original)
    }

    pub fn to_compression_quality(&self) -> Option<CompressionQuality> {
        match self {
            CompressionLevel::Original => None,
            CompressionLevel::High => Some(CompressionQuality::High),
            CompressionLevel::Medium => Some(CompressionQuality::Medium),
            CompressionLevel::Low => Some(CompressionQuality::Low),
            CompressionLevel::Minimal => Some(CompressionQuality::Minimal),
        }
    }

    pub fn all() -> Vec<CompressionLevel> {
        vec![
            CompressionLevel::Original,
            CompressionLevel::High,
            CompressionLevel::Medium,
            CompressionLevel::Low,
            CompressionLevel::Minimal,
        ]
    }
}

/// Compression quality levels (for backend compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionQuality {
    High,
    Medium,
    Low,
    Minimal,
}

impl CompressionQuality {
    pub fn percentage(&self) -> u8 {
        match self {
            CompressionQuality::High => 90,
            CompressionQuality::Medium => 50,
            CompressionQuality::Low => 30,
            CompressionQuality::Minimal => 10,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CompressionQuality::High => "High quality, smaller file",
            CompressionQuality::Medium => "Balanced quality/size",
            CompressionQuality::Low => "Lower quality, smaller file",
            CompressionQuality::Minimal => "Lowest quality, minimal file size",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CompressionQuality::High => "90% Quality",
            CompressionQuality::Medium => "50% Quality",
            CompressionQuality::Low => "30% Quality",
            CompressionQuality::Minimal => "10% Quality",
        }
    }

    pub fn all() -> Vec<CompressionQuality> {
        vec![
            CompressionQuality::High,
            CompressionQuality::Medium,
            CompressionQuality::Low,
            CompressionQuality::Minimal,
        ]
    }
}

/// Compression estimate data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressionEstimate {
    pub original_size_mb: u64,
    pub estimated_size_mb: u64,
    pub compression_ratio: f64,
    pub estimated_time_seconds: u64,
    pub quality_used: CompressionQuality,
    pub size_reduction_mb: u64,
    pub size_reduction_percentage: u8,
}

/// Compression level selector component
#[component]
pub fn CompressionLevelSelector(
    original_size_mb: u64,
    selected_level: CompressionLevel,
    on_level_change: impl Fn(CompressionLevel) + 'static,
    compression_available: bool,
) -> impl IntoView {
    let (dropdown_open, set_dropdown_open) = signal(false);

    let toggle_dropdown = move |_| {
        set_dropdown_open.update(|open| *open = !*open);
    };

    let close_dropdown = move |_| {
        set_dropdown_open.set(false);
    };

    html! {
        <div class="relative">
            <button
                class={format!("flex items-center justify-between w-full px-3 py-2 text-sm border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-colors {} {}",
                    props.selected_level.color_classes(),
                    props.selected_level.hover_color_classes()
                )}
                onclick={toggle_dropdown}
            >
                <div class="flex items-center space-x-2">
                    <span class="text-lg leading-none">
                        {props.selected_level.intensity_indicator()}
                    </span>
                    <span class="font-medium">
                        {props.selected_level.label()}
                    </span>
                </div>
                <svg class={format!("w-4 h-4 transition-transform {}",
                    if *dropdown_open { "rotate-180" } else { "" }
                )} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                </svg>
            </button>

            {
                if *dropdown_open {
                    html! {
                        <>
                            <div class="fixed inset-0 z-10" onclick={close_dropdown}></div>
                            <div class="absolute z-20 w-full mt-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg shadow-lg">
                                <div class="py-1">
                                    {
                                        CompressionLevel::all().into_iter().map(|level| {
                                            let is_selected = level == props.selected_level;
                                            let is_disabled = level.is_compressed() && !props.compression_available;
                                            let on_level_change = props.on_level_change.clone();

                                            html! {
                                                <button
                                                    key={format!("{:?}", level)}
                                                    class={
                                                        if is_disabled {
                                                            "w-full px-3 py-2 text-left transition-colors border-l-4 opacity-50 cursor-not-allowed bg-gray-50 dark:bg-gray-800 border-l-gray-300 dark:border-l-gray-600".to_string()
                                                        } else if is_selected {
                                                            format!("w-full px-3 py-2 text-left transition-colors border-l-4 {}", level.color_classes())
                                                        } else {
                                                            format!("w-full px-3 py-2 text-left transition-colors border-l-4 border-l-transparent {} {}", level.color_classes(), level.hover_color_classes())
                                                        }
                                                    }
                                                    onclick={
                                                        if !is_disabled {
                                                            let dropdown_open = dropdown_open.clone();
                                                            Callback::from(move |_| {
                                                                on_level_change.emit(level);
                                                                dropdown_open.set(false);
                                                            })
                                                        } else {
                                                            Callback::from(|_| {})
                                                        }
                                                    }
                                                    disabled={is_disabled}
                                                >
                                                    <div class="flex items-center space-x-3">
                                                        <span class="text-lg leading-none">
                                                            {level.intensity_indicator()}
                                                        </span>
                                                        <div class="flex-1">
                                                            <div class="font-medium">
                                                                {level.label()}
                                                                {
                                                                    if is_disabled {
                                                                        html! { <span class="text-xs opacity-60 ml-1">{"(FFmpeg required)"}</span> }
                                                                    } else {
                                                                        html! {}
                                                                    }
                                                                }
                                                            </div>
                                                            <div class="text-xs opacity-75 mt-1">
                                                                {level.description()}
                                                            </div>
                                                        </div>
                                                    </div>
                                                </button>
                                            }
                                        }).collect::<Html>()
                                    }
                                </div>
                            </div>
                        </>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

/// Compression settings component props
#[derive(Properties, PartialEq)]
pub struct CompressionSettingsProps {
    pub original_size_mb: u64,
    pub original_resolution: String,
    pub enabled: bool,
    pub selected_quality: CompressionQuality,
    pub on_enabled_change: Callback<bool>,
    pub on_quality_change: Callback<CompressionQuality>,
    pub on_estimate_update: Callback<Option<CompressionEstimate>>,
}

/// Compression settings component
#[function_component(CompressionSettings)]
pub fn compression_settings(props: &CompressionSettingsProps) -> Html {
    console::log_1(&format!("🎛️ Rendering CompressionSettings component with original_size_mb: {}", props.original_size_mb).into());
    let compression_available = use_state(|| None::<bool>);
    let estimate = use_state(|| None::<CompressionEstimate>);
    let loading_estimate = use_state(|| false);

    // Check compression availability on mount
    {
        let compression_available = compression_available.clone();
        use_effect_with((), move |_| {
            console::log_1(&"🔧 Checking compression availability...".into());
            spawn_local(async move {
                match tauri_api::check_compression_availability().await {
                    Ok(result) => {
                        console::log_1(&format!("✅ Compression availability check result: {}", result.data).into());
                        compression_available.set(Some(result.data));
                        if !result.data {
                            console::log_1(&"⚠️ FFmpeg not available for compression".into());
                        } else {
                            console::log_1(&"✅ FFmpeg is available for compression".into());
                        }
                    }
                    Err(e) => {
                        console::log_1(&format!("❌ Failed to check compression availability: {}", e.error).into());
                        compression_available.set(Some(false));
                    }
                }
            });
            || ()
        });
    }

    // Update estimate when quality changes
    {
        let estimate = estimate.clone();
        let loading_estimate = loading_estimate.clone();
        let on_estimate_update = props.on_estimate_update.clone();
        let original_size_mb = props.original_size_mb;
        let selected_quality = props.selected_quality;
        let enabled = props.enabled;

        use_effect_with((selected_quality, enabled), move |(quality, enabled)| {
            if *enabled && original_size_mb > 0 {
                loading_estimate.set(true);
                let estimate = estimate.clone();
                let loading_estimate = loading_estimate.clone();
                let on_estimate_update = on_estimate_update.clone();
                let quality = *quality;

                spawn_local(async move {
                    match tauri_api::estimate_compression_size(original_size_mb, quality).await {
                        Ok(result) => {
                            let estimate_data = result.data.clone();
                            estimate.set(Some(estimate_data.clone()));
                            on_estimate_update.emit(Some(estimate_data.clone()));
                            console::log_1(&format!("✅ Compression estimate updated: {} MB -> {} MB",
                                original_size_mb, estimate_data.estimated_size_mb).into());
                        }
                        Err(e) => {
                            console::log_1(&format!("❌ Failed to estimate compression: {}", e.error).into());
                            estimate.set(None);
                            on_estimate_update.emit(None);
                        }
                    }
                    loading_estimate.set(false);
                });
            } else {
                estimate.set(None);
                on_estimate_update.emit(None);
            }
            || ()
        });
    }

    // Don't render if compression is not available
    if let Some(false) = *compression_available {
        return html! {
            <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
                <div class="flex items-center space-x-2">
                    <svg class="w-5 h-5 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"></path>
                    </svg>
                    <div>
                        <h3 class="text-sm font-medium text-yellow-800 dark:text-yellow-200">
                            {"Compression Not Available"}
                        </h3>
                        <p class="text-sm text-yellow-700 dark:text-yellow-300 mt-1">
                            {"FFmpeg is required for video compression. Please install FFmpeg to use this feature."}
                        </p>
                    </div>
                </div>
            </div>
        };
    }

    // Show loading state while checking availability
    if compression_available.is_none() {
        return html! {
            <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
                <div class="flex items-center space-x-2">
                    <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
                    <span class="text-sm text-gray-600 dark:text-gray-400">{"Checking compression availability..."}</span>
                </div>
            </div>
        };
    }

    html! {
        <div class="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700">
            <div class="p-6 border-b border-gray-200 dark:border-gray-700">
                <div class="flex items-center justify-between">
                    <div>
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                            {"Video Compression"}
                        </h3>
                        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                            {"Reduce file size while maintaining quality"}
                        </p>
                    </div>
                    <label class="relative inline-flex items-center cursor-pointer">
                        <input
                            type="checkbox"
                            class="sr-only peer"
                            checked={props.enabled}
                            onchange={
                                let on_enabled_change = props.on_enabled_change.clone();
                                Callback::from(move |e: Event| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    on_enabled_change.emit(input.checked());
                                })
                            }
                        />
                        <div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
                        <span class="ml-3 text-sm font-medium text-gray-900 dark:text-gray-300">
                            {if props.enabled { "Enabled" } else { "Disabled" }}
                        </span>
                    </label>
                </div>
            </div>

            {
                if props.enabled {
                    html! {
                        <div class="p-6 space-y-6">
                            // Quality selector
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                                    {"Compression Quality"}
                                </label>
                                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
                                    {
                                        CompressionQuality::all().into_iter().map(|quality| {
                                            let is_selected = quality == props.selected_quality;
                                            let on_quality_change = props.on_quality_change.clone();
                                            
                                            html! {
                                                <button
                                                    key={format!("{:?}", quality)}
                                                    class={format!("p-3 rounded-lg border-2 text-left transition-all {}",
                                                        if is_selected {
                                                            "border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                                                        } else {
                                                            "border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
                                                        }
                                                    )}
                                                    onclick={
                                                        Callback::from(move |_| {
                                                            on_quality_change.emit(quality);
                                                        })
                                                    }
                                                >
                                                    <div class="font-medium text-sm text-gray-900 dark:text-white">
                                                        {quality.label()}
                                                    </div>
                                                    <div class="text-xs text-gray-600 dark:text-gray-400 mt-1">
                                                        {quality.description()}
                                                    </div>
                                                </button>
                                            }
                                        }).collect::<Html>()
                                    }
                                </div>
                            </div>

                            // Compression estimate
                            {
                                if let Some(est) = &*estimate {
                                    html! {
                                        <CompressionEstimateDisplay 
                                            estimate={est.clone()}
                                            original_resolution={props.original_resolution.clone()}
                                        />
                                    }
                                } else if *loading_estimate {
                                    html! {
                                        <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-4">
                                            <div class="flex items-center space-x-2">
                                                <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
                                                <span class="text-sm text-gray-600 dark:text-gray-400">{"Calculating compression estimate..."}</span>
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

/// Compression estimate display component props
#[derive(Properties, PartialEq)]
pub struct CompressionEstimateDisplayProps {
    pub estimate: CompressionEstimate,
    pub original_resolution: String,
}

/// Compression estimate display component
#[function_component(CompressionEstimateDisplay)]
pub fn compression_estimate_display(props: &CompressionEstimateDisplayProps) -> Html {
    let estimate = &props.estimate;
    
    // Format time estimate
    let time_text = if estimate.estimated_time_seconds < 60 {
        format!("~{} seconds", estimate.estimated_time_seconds)
    } else if estimate.estimated_time_seconds < 3600 {
        format!("~{} minutes", estimate.estimated_time_seconds / 60)
    } else {
        format!("~{} hours", estimate.estimated_time_seconds / 3600)
    };

    html! {
        <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
            <h4 class="text-sm font-medium text-blue-800 dark:text-blue-200 mb-3">
                {"Compression Estimate"}
            </h4>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                <div>
                    <p class="text-blue-700 dark:text-blue-300 font-medium">{"File Size"}</p>
                    <p class="text-blue-600 dark:text-blue-400">
                        {format!("{} MB → {} MB", estimate.original_size_mb, estimate.estimated_size_mb)}
                    </p>
                    <p class="text-xs text-blue-500 dark:text-blue-500">
                        {format!("{}% reduction", estimate.size_reduction_percentage)}
                    </p>
                </div>
                <div>
                    <p class="text-blue-700 dark:text-blue-300 font-medium">{"Resolution"}</p>
                    <p class="text-blue-600 dark:text-blue-400">
                        {&props.original_resolution}
                    </p>
                    <p class="text-xs text-blue-500 dark:text-blue-500">
                        {"Maintained"}
                    </p>
                </div>
                <div>
                    <p class="text-blue-700 dark:text-blue-300 font-medium">{"Processing Time"}</p>
                    <p class="text-blue-600 dark:text-blue-400">
                        {time_text}
                    </p>
                    <p class="text-xs text-blue-500 dark:text-blue-500">
                        {"Estimated"}
                    </p>
                </div>
            </div>
        </div>
    }
}
