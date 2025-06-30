use yew::prelude::*;
use web_sys::MouseEvent;
use crate::services::tauri_api::{BatchItem, BatchItemStatus};

#[derive(Properties, PartialEq)]
pub struct BatchQueueViewProps {
    pub items: Vec<BatchItem>,
    pub on_item_action: Option<Callback<(String, ItemAction)>>,
    pub on_bulk_action: Option<Callback<BulkAction>>,
}

#[derive(Clone, PartialEq)]
pub enum ItemAction {
    Pause,
    Resume,
    Retry,
    Cancel,
    ShowDetails,
    OpenFolder,
    CopyPath,
}

#[derive(Clone, PartialEq)]
pub enum BulkAction {
    RetryFailed,
    PauseAll,
    ResumeAll,
    CancelAll,
    RemoveCompleted,
    SelectAll,
    SelectNone,
}

#[function_component(BatchQueueView)]
pub fn batch_queue_view(props: &BatchQueueViewProps) -> Html {
    let selected_items = use_state(|| std::collections::HashSet::<String>::new());

    // Reset selection state when items change significantly (new batch started)
    // but preserve selection during normal updates (status changes, progress updates)
    {
        let selected_items = selected_items.clone();
        let current_items = props.items.clone();
        use_effect_with(props.items.len(), move |_| {
            // Only reset if we have a completely different set of items
            // Check if any of the currently selected items still exist
            let current_ids: std::collections::HashSet<String> = current_items.iter().map(|item| item.id.clone()).collect();
            let selected_clone = (*selected_items).clone();

            // If none of the selected items exist in the current batch, reset selection
            if !selected_clone.is_empty() && selected_clone.is_disjoint(&current_ids) {
                selected_items.set(std::collections::HashSet::new());
            }
            || {}
        });
    }

    if props.items.is_empty() {
        return html! {
            <div class="text-center py-12 text-gray-500 dark:text-gray-400">
                <div class="text-6xl mb-4">{"📥"}</div>
                <h3 class="text-lg font-medium mb-2">{"No items in queue"}</h3>
                <p class="text-sm">{"Add some URLs to get started with batch processing"}</p>
            </div>
        };
    }

    // Group items by status
    let mut grouped_items: std::collections::HashMap<BatchItemStatus, Vec<&BatchItem>> = 
        std::collections::HashMap::new();
    
    for item in &props.items {
        grouped_items.entry(item.status.clone()).or_insert_with(Vec::new).push(item);
    }

    // Calculate summary stats
    let total_items = props.items.len();
    let completed_items = grouped_items.get(&BatchItemStatus::Completed).map_or(0, |v| v.len());
    let failed_items = grouped_items.get(&BatchItemStatus::Failed).map_or(0, |v| v.len());
    let processing_items = grouped_items.get(&BatchItemStatus::Downloading).map_or(0, |v| v.len()) +
                          grouped_items.get(&BatchItemStatus::Extracting).map_or(0, |v| v.len()) +
                          grouped_items.get(&BatchItemStatus::Compressing).map_or(0, |v| v.len());

    let progress_percentage = if total_items > 0 {
        (completed_items as f32 / total_items as f32) * 100.0
    } else {
        0.0
    };

    // Define status order and styling
    let status_groups = vec![
        (BatchItemStatus::Downloading, "🔵", "Processing", "border-blue-500 bg-blue-50 dark:bg-blue-900/20", true),
        (BatchItemStatus::Extracting, "🟡", "Extracting", "border-yellow-500 bg-yellow-50 dark:bg-yellow-900/20", true),
        (BatchItemStatus::Compressing, "🟣", "Compressing", "border-purple-500 bg-purple-50 dark:bg-purple-900/20", true),
        (BatchItemStatus::Queued, "⏳", "Queued", "border-gray-500 bg-gray-50 dark:bg-gray-900/20", true),
        (BatchItemStatus::Validating, "🔍", "Validating", "border-gray-500 bg-gray-50 dark:bg-gray-900/20", true),
        (BatchItemStatus::Completed, "🟢", "Completed", "border-green-500 bg-green-50 dark:bg-green-900/20", false),
        (BatchItemStatus::Failed, "🔴", "Failed", "border-red-500 bg-red-50 dark:bg-red-900/20", false),
        (BatchItemStatus::Skipped, "⚪", "Skipped", "border-gray-400 bg-gray-50 dark:bg-gray-900/20", false),
        (BatchItemStatus::Cancelled, "⚫", "Cancelled", "border-gray-600 bg-gray-50 dark:bg-gray-900/20", false),
    ];

    let on_select_all = {
        let selected_items = selected_items.clone();
        let items = props.items.clone();
        Callback::from(move |_| {
            let all_ids: std::collections::HashSet<String> = items.iter().map(|item| item.id.clone()).collect();
            selected_items.set(all_ids);
        })
    };

    let on_select_none = {
        let selected_items = selected_items.clone();
        Callback::from(move |_| {
            selected_items.set(std::collections::HashSet::new());
        })
    };

    // Calculate selection state more reliably
    let current_item_ids: std::collections::HashSet<String> = props.items.iter().map(|item| item.id.clone()).collect();
    let valid_selected_items: std::collections::HashSet<String> = selected_items.iter()
        .filter(|id| current_item_ids.contains(*id))
        .cloned()
        .collect();
    let is_all_selected = !props.items.is_empty() && valid_selected_items.len() == props.items.len();

    html! {
        <div class="space-y-6">
            // Bulk Actions Panel
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-4 border border-gray-200 dark:border-gray-700">
                <div class="flex flex-col lg:flex-row lg:items-center lg:justify-between space-y-4 lg:space-y-0">
                    // Selection Controls
                    <div class="flex items-center space-x-4">
                        <div class="flex items-center space-x-2">
                            <input
                                type="checkbox"
                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                checked={is_all_selected}
                                onchange={if is_all_selected { on_select_none } else { on_select_all }}
                            />
                            <span class="text-sm text-gray-700 dark:text-gray-300">
                                {format!("Select All ({}/{})", valid_selected_items.len(), props.items.len())}
                            </span>
                        </div>
                        
                        {if !valid_selected_items.is_empty() {
                            html! {
                                <div class="flex items-center space-x-2">
                                    <button class="px-3 py-1 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-md transition-colors duration-200">
                                        {"Retry Selected"}
                                    </button>
                                    <button class="px-3 py-1 text-sm bg-gray-600 hover:bg-gray-700 text-white rounded-md transition-colors duration-200">
                                        {"Cancel Selected"}
                                    </button>
                                </div>
                            }
                        } else {
                            html! {}
                        }}
                    </div>

                    // Quick Actions
                    <div class="flex items-center space-x-2">
                        {if failed_items > 0 {
                            html! {
                                <button class="px-3 py-1 text-sm bg-red-600 hover:bg-red-700 text-white rounded-md transition-colors duration-200">
                                    {format!("Retry Failed ({})", failed_items)}
                                </button>
                            }
                        } else {
                            html! {}
                        }}
                        
                        {if completed_items > 0 {
                            html! {
                                <button class="px-3 py-1 text-sm bg-green-600 hover:bg-green-700 text-white rounded-md transition-colors duration-200">
                                    {"Remove Completed"}
                                </button>
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                </div>

                // Overall Progress
                <div class="mt-4">
                    <div class="flex justify-between text-sm text-gray-600 dark:text-gray-400 mb-2">
                        <span>{format!("Progress: {}/{} completed", completed_items, total_items)}</span>
                        <span>{format!("{:.1}%", progress_percentage)}</span>
                    </div>
                    <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3">
                        <div 
                            class="bg-gradient-to-r from-blue-500 to-green-500 h-3 rounded-full transition-all duration-500"
                            style={format!("width: {}%", progress_percentage)}
                        ></div>
                    </div>
                    <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
                        <span>{format!("Processing: {}", processing_items)}</span>
                        {if failed_items > 0 {
                            html! { <span class="text-red-600 dark:text-red-400">{format!("Failed: {}", failed_items)}</span> }
                        } else {
                            html! {}
                        }}
                    </div>
                </div>
            </div>

            // Status Groups
            <div class="space-y-4">
                {for status_groups.iter().filter_map(|(status, icon, label, style, expanded)| {
                    grouped_items.get(status).map(|items| {
                        let count = items.len();
                        let items_owned: Vec<BatchItem> = items.iter().map(|&item| item.clone()).collect();

                        html! {
                            <StatusGroup
                                status={status.clone()}
                                icon={icon}
                                label={label}
                                style={style}
                                items={items_owned}
                                count={count}
                                expanded={*expanded}
                                selected_items={valid_selected_items.clone()}
                                on_item_select={
                                    let selected_items = selected_items.clone();
                                    Callback::from(move |item_id: String| {
                                        let mut new_selected = (*selected_items).clone();
                                        if new_selected.contains(&item_id) {
                                            new_selected.remove(&item_id);
                                        } else {
                                            new_selected.insert(item_id);
                                        }
                                        selected_items.set(new_selected);
                                    })
                                }
                                on_item_action={props.on_item_action.clone()}
                            />
                        }
                    })
                })}
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct StatusGroupProps {
    pub status: BatchItemStatus,
    pub icon: &'static str,
    pub label: &'static str,
    pub style: &'static str,
    pub items: Vec<BatchItem>,
    pub count: usize,
    pub expanded: bool,
    pub selected_items: std::collections::HashSet<String>,
    pub on_item_select: Callback<String>,
    pub on_item_action: Option<Callback<(String, ItemAction)>>,
}

#[function_component(StatusGroup)]
pub fn status_group(props: &StatusGroupProps) -> Html {
    // Use only local state for expansion control to avoid conflicts
    // Initialize with props.expanded but then manage independently
    let is_expanded = use_state(|| props.expanded);

    let toggle_expanded = {
        let is_expanded = is_expanded.clone();
        Callback::from(move |_| {
            is_expanded.set(!*is_expanded);
        })
    };

    html! {
        <div class={format!("bg-white dark:bg-gray-800 rounded-lg shadow-md border-l-4 {}", props.style)}>
            <div class="p-4">
                // Clickable header with dropdown arrow
                <div
                    class="flex items-center justify-between cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 -m-2 p-2 rounded-md transition-colors duration-200"
                    onclick={toggle_expanded}
                >
                    <div class="flex items-center space-x-2 text-lg font-semibold text-gray-800 dark:text-white">
                        <span class="text-xl">{props.icon}</span>
                        <span>{format!("{} ({})", props.label, props.count)}</span>
                        // Dropdown arrow
                        <span class={format!("text-sm text-gray-500 dark:text-gray-400 transition-transform duration-200 {}",
                            if *is_expanded { "rotate-180" } else { "rotate-0" }
                        )}>
                            {"▼"}
                        </span>
                    </div>

                    <div class="flex items-center space-x-2">
                        {if props.status == BatchItemStatus::Failed && props.count > 0 {
                            html! {
                                <button
                                    class="px-3 py-1 text-sm bg-red-600 hover:bg-red-700 text-white rounded-md transition-colors duration-200"
                                    onclick={|e: MouseEvent| {
                                        e.stop_propagation(); // Prevent header toggle when clicking retry
                                    }}
                                >
                                    {"Retry All"}
                                </button>
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                </div>

                // Collapsible content section
                {if *is_expanded {
                    if props.items.is_empty() {
                        html! {
                            <div class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-600">
                                <div class="text-center py-8 text-gray-500 dark:text-gray-400">
                                    <p class="text-sm">{"No items in this category"}</p>
                                </div>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-600">
                                <div class="max-h-96 overflow-y-auto overflow-x-hidden batch-scroll-container scrollbar-thin scrollbar-thumb-gray-400 scrollbar-track-gray-200 dark:scrollbar-thumb-gray-600 dark:scrollbar-track-gray-800 pr-2">
                                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-3 p-1">
                                        {for props.items.iter().map(|item| {
                                            html! {
                                                <CompactItemCard
                                                    key={item.id.clone()}
                                                    item={item.clone()}
                                                    selected={props.selected_items.contains(&item.id)}
                                                    on_select={
                                                        let on_item_select = props.on_item_select.clone();
                                                        let item_id = item.id.clone();
                                                        Callback::from(move |_| on_item_select.emit(item_id.clone()))
                                                    }
                                                    on_action={props.on_item_action.clone()}
                                                />
                                            }
                                        })}
                                    </div>
                                </div>
                            </div>
                        }
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ItemCardProps {
    pub item: BatchItem,
    pub selected: bool,
    pub on_select: Callback<()>,
    pub on_action: Option<Callback<(String, ItemAction)>>,
}

#[function_component(ItemCard)]
pub fn item_card(props: &ItemCardProps) -> Html {
    let item = &props.item;

    let (status_icon, action_buttons) = match item.status {
        BatchItemStatus::Completed => ("✅", html! {
            <div class="flex space-x-1">
                <button
                    class="p-1 text-gray-500 hover:text-blue-600 transition-colors rounded"
                    title="Open folder"
                    onclick={
                        let on_action = props.on_action.clone();
                        let item_id = item.id.clone();
                        Callback::from(move |_| {
                            if let Some(callback) = &on_action {
                                callback.emit((item_id.clone(), ItemAction::OpenFolder));
                            }
                        })
                    }
                >
                    {"📁"}
                </button>
                <button
                    class="p-1 text-gray-500 hover:text-blue-600 transition-colors rounded"
                    title="Copy path"
                    onclick={
                        let on_action = props.on_action.clone();
                        let item_id = item.id.clone();
                        Callback::from(move |_| {
                            if let Some(callback) = &on_action {
                                callback.emit((item_id.clone(), ItemAction::CopyPath));
                            }
                        })
                    }
                >
                    {"📋"}
                </button>
            </div>
        }),
        BatchItemStatus::Failed => ("❌", html! {
            <div class="flex space-x-1">
                <button
                    class="p-1 text-gray-500 hover:text-blue-600 transition-colors rounded"
                    title="Retry"
                    onclick={
                        let on_action = props.on_action.clone();
                        let item_id = item.id.clone();
                        Callback::from(move |_| {
                            if let Some(callback) = &on_action {
                                callback.emit((item_id.clone(), ItemAction::Retry));
                            }
                        })
                    }
                >
                    {"🔄"}
                </button>
                <button
                    class="p-1 text-gray-500 hover:text-blue-600 transition-colors rounded"
                    title="Show details"
                    onclick={
                        let on_action = props.on_action.clone();
                        let item_id = item.id.clone();
                        Callback::from(move |_| {
                            if let Some(callback) = &on_action {
                                callback.emit((item_id.clone(), ItemAction::ShowDetails));
                            }
                        })
                    }
                >
                    {"ℹ️"}
                </button>
            </div>
        }),
        BatchItemStatus::Downloading | BatchItemStatus::Extracting | BatchItemStatus::Compressing => {
            ("⏳", html! {
                <div class="flex space-x-1">
                    <button
                        class="p-1 text-gray-500 hover:text-yellow-600 transition-colors rounded"
                        title="Pause"
                        onclick={
                            let on_action = props.on_action.clone();
                            let item_id = item.id.clone();
                            Callback::from(move |_| {
                                if let Some(callback) = &on_action {
                                    callback.emit((item_id.clone(), ItemAction::Pause));
                                }
                            })
                        }
                    >
                        {"⏸️"}
                    </button>
                    <button
                        class="p-1 text-gray-500 hover:text-red-600 transition-colors rounded"
                        title="Cancel"
                        onclick={
                            let on_action = props.on_action.clone();
                            let item_id = item.id.clone();
                            Callback::from(move |_| {
                                if let Some(callback) = &on_action {
                                    callback.emit((item_id.clone(), ItemAction::Cancel));
                                }
                            })
                        }
                    >
                        {"❌"}
                    </button>
                </div>
            })
        },
        _ => ("⏳", html! {}),
    };

    // Extract title and create display URL
    let title = if let Some(video_info) = &item.video_info {
        video_info.title.clone()
    } else {
        "Loading video information...".to_string()
    };

    let display_url = if item.url.len() > 40 {
        format!("{}...", &item.url[..37])
    } else {
        item.url.clone()
    };

    html! {
        <div class={format!(
            "bg-white dark:bg-gray-800 rounded-lg border shadow-sm hover:shadow-md transition-all duration-200 p-4 {}",
            if props.selected {
                "border-blue-500 bg-blue-50 dark:bg-blue-900/20"
            } else {
                "border-gray-200 dark:border-gray-700"
            }
        )}>
            // Header section with checkbox, status, and actions
            <div class="flex items-start justify-between mb-3">
                <div class="flex items-start space-x-3 flex-1 min-w-0">
                    <input
                        type="checkbox"
                        class="rounded border-gray-300 text-blue-600 focus:ring-blue-500 mt-1"
                        checked={props.selected}
                        onchange={let on_select = props.on_select.clone(); Callback::from(move |_| on_select.emit(()))}
                    />
                    <div class="flex items-center space-x-2">
                        <span class="text-xl">{status_icon}</span>
                        <div class={format!("text-xs font-medium px-2 py-1 rounded-full {}",
                            match item.status {
                                BatchItemStatus::Completed => "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400",
                                BatchItemStatus::Failed => "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400",
                                BatchItemStatus::Downloading | BatchItemStatus::Extracting | BatchItemStatus::Compressing =>
                                    "bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400",
                                _ => "bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400",
                            }
                        )}>
                            {match item.status {
                                BatchItemStatus::Queued => "Queued",
                                BatchItemStatus::Validating => "Validating",
                                BatchItemStatus::Extracting => "Extracting",
                                BatchItemStatus::Downloading => "Downloading",
                                BatchItemStatus::Compressing => "Compressing",
                                BatchItemStatus::Completed => "Completed",
                                BatchItemStatus::Failed => "Failed",
                                BatchItemStatus::Skipped => "Skipped",
                                BatchItemStatus::Cancelled => "Cancelled",
                            }}
                        </div>
                    </div>
                </div>
                <div class="flex-shrink-0">
                    {action_buttons}
                </div>
            </div>

            // Content section with title and URL
            <div class="mb-3">
                <div class="text-sm font-medium text-gray-900 dark:text-white mb-1 leading-tight">
                    <div class="line-clamp-2" title={title.clone()}>
                        {if title.len() > 80 { format!("{}...", &title[..77]) } else { title }}
                    </div>
                </div>
                <a
                    href={item.url.clone()}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-xs text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 hover:underline transition-colors truncate block cursor-pointer flex items-center space-x-1"
                    title={format!("Click to open original Facebook link: {}", item.url)}
                    onclick={|e: MouseEvent| {
                        e.stop_propagation(); // Prevent card selection when clicking URL
                    }}
                >
                    <span>{display_url}</span>
                    <span class="text-xs opacity-60">{"🔗"}</span>
                </a>
            </div>

            {if let Some(progress) = &item.download_progress {
                // Fix progress calculation - backend stores as 0-100, not 0-1
                // Ensure it's between 0-100% and handle edge cases
                let progress_percentage = if progress.progress > 1.0 {
                    // Already a percentage (0-100)
                    progress.progress.min(100.0).max(0.0)
                } else {
                    // Decimal format (0-1), convert to percentage
                    (progress.progress * 100.0).min(100.0).max(0.0)
                };

                html! {
                    <div class="mb-2">
                        <div class="flex justify-between text-xs text-gray-600 dark:text-gray-400 mb-1">
                            <span>{format!("{:.1}%", progress_percentage)}</span>
                            {if let Some(speed) = progress.speed_bytes_per_sec {
                                html! {
                                    <span>{format!("{:.1} MB/s", speed as f64 / 1024.0 / 1024.0)}</span>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                        <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2">
                            <div
                                class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                                style={format!("width: {}%", progress_percentage)}
                            ></div>
                        </div>
                        {if let Some(eta) = progress.eta_seconds {
                            html! {
                                <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                    {format!("ETA: {}m {}s", eta / 60, eta % 60)}
                                </div>
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                }
            } else {
                html! {}
            }}

            {if let Some(error) = &item.error_message {
                html! {
                    <div class="mt-2 p-2 bg-red-50 dark:bg-red-900/20 rounded border border-red-200 dark:border-red-800">
                        <div class="text-xs text-red-700 dark:text-red-300 break-words">
                            {if error.len() > 200 {
                                format!("{}...", &error[..197])
                            } else {
                                error.clone()
                            }}
                        </div>
                        {if error.contains("Video metadata was extracted successfully") {
                            html! {
                                <div class="mt-1 text-xs text-amber-600 dark:text-amber-400">
                                    {"💡 The video information was found but download streams are not accessible. Try accessing the video directly on Facebook to verify it's publicly available."}
                                </div>
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                }
            } else {
                html! {}
            }}
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct CompactItemCardProps {
    pub item: BatchItem,
    pub selected: bool,
    pub on_select: Callback<()>,
    pub on_action: Option<Callback<(String, ItemAction)>>,
}

#[function_component(CompactItemCard)]
pub fn compact_item_card(props: &CompactItemCardProps) -> Html {
    let item = &props.item;

    let (status_icon, action_buttons) = match item.status {
        BatchItemStatus::Completed => ("✅", html! {
            <div class="flex space-x-1">
                <button
                    class="p-1 text-xs text-gray-500 hover:text-blue-600 transition-colors rounded"
                    title="Open folder"
                    onclick={
                        let on_action = props.on_action.clone();
                        let item_id = item.id.clone();
                        Callback::from(move |_| {
                            if let Some(callback) = &on_action {
                                callback.emit((item_id.clone(), ItemAction::OpenFolder));
                            }
                        })
                    }
                >
                    {"📁"}
                </button>
            </div>
        }),
        BatchItemStatus::Failed => ("❌", html! {
            <div class="flex space-x-1">
                <button
                    class="p-1 text-xs text-gray-500 hover:text-blue-600 transition-colors rounded"
                    title="Retry"
                    onclick={
                        let on_action = props.on_action.clone();
                        let item_id = item.id.clone();
                        Callback::from(move |_| {
                            if let Some(callback) = &on_action {
                                callback.emit((item_id.clone(), ItemAction::Retry));
                            }
                        })
                    }
                >
                    {"🔄"}
                </button>
            </div>
        }),
        BatchItemStatus::Downloading | BatchItemStatus::Extracting | BatchItemStatus::Compressing => {
            ("⏳", html! {
                <div class="flex space-x-1">
                    <button
                        class="p-1 text-xs text-gray-500 hover:text-yellow-600 transition-colors rounded"
                        title="Pause"
                        onclick={
                            let on_action = props.on_action.clone();
                            let item_id = item.id.clone();
                            Callback::from(move |_| {
                                if let Some(callback) = &on_action {
                                    callback.emit((item_id.clone(), ItemAction::Pause));
                                }
                            })
                        }
                    >
                        {"⏸️"}
                    </button>
                    <button
                        class="p-1 text-xs text-gray-500 hover:text-red-600 transition-colors rounded"
                        title="Cancel"
                        onclick={
                            let on_action = props.on_action.clone();
                            let item_id = item.id.clone();
                            Callback::from(move |_| {
                                if let Some(callback) = &on_action {
                                    callback.emit((item_id.clone(), ItemAction::Cancel));
                                }
                            })
                        }
                    >
                        {"❌"}
                    </button>
                </div>
            })
        },
        _ => ("⏳", html! {}),
    };

    // Extract title and create display URL
    let title = if let Some(video_info) = &item.video_info {
        video_info.title.clone()
    } else {
        "Loading video information...".to_string()
    };

    let display_url = if item.url.len() > 25 {
        format!("{}...", &item.url[..22])
    } else {
        item.url.clone()
    };

    html! {
        <div class={format!(
            "bg-white dark:bg-gray-800 rounded-lg border shadow-sm hover:shadow-md transition-all duration-200 p-3 h-32 flex flex-col {}",
            if props.selected {
                "border-blue-500 bg-blue-50 dark:bg-blue-900/20"
            } else {
                "border-gray-200 dark:border-gray-700"
            }
        )}>
            // Header with checkbox, status icon, and actions
            <div class="flex items-center justify-between mb-2">
                <div class="flex items-center space-x-2">
                    <input
                        type="checkbox"
                        class="rounded border-gray-300 text-blue-600 focus:ring-blue-500 w-3 h-3"
                        checked={props.selected}
                        onchange={let on_select = props.on_select.clone(); Callback::from(move |_| on_select.emit(()))}
                    />
                    <span class="text-lg">{status_icon}</span>
                    <div class={format!("text-xs font-medium px-1 py-0.5 rounded {}",
                        match item.status {
                            BatchItemStatus::Completed => "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400",
                            BatchItemStatus::Failed => "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400",
                            BatchItemStatus::Downloading | BatchItemStatus::Extracting | BatchItemStatus::Compressing =>
                                "bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400",
                            _ => "bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400",
                        }
                    )}>
                        {match item.status {
                            BatchItemStatus::Queued => "Queue",
                            BatchItemStatus::Validating => "Valid",
                            BatchItemStatus::Extracting => "Extract",
                            BatchItemStatus::Downloading => "Download",
                            BatchItemStatus::Compressing => "Compress",
                            BatchItemStatus::Completed => "Done",
                            BatchItemStatus::Failed => "Failed",
                            BatchItemStatus::Skipped => "Skip",
                            BatchItemStatus::Cancelled => "Cancel",
                        }}
                    </div>
                </div>
                <div class="flex-shrink-0">
                    {action_buttons}
                </div>
            </div>

            // Title and URL
            <div class="flex-1 min-h-0 mb-2">
                <div class="text-xs font-medium text-gray-900 dark:text-white mb-1 line-clamp-2 leading-tight" title={title.clone()}>
                    {if title.len() > 40 { format!("{}...", &title[..37]) } else { title }}
                </div>
                <a
                    href={item.url.clone()}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-xs text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 hover:underline transition-colors truncate block cursor-pointer flex items-center space-x-1"
                    title={format!("Click to open original Facebook link: {}", item.url)}
                    onclick={|e: MouseEvent| {
                        e.stop_propagation(); // Prevent card selection when clicking URL
                    }}
                >
                    <span class="truncate">{display_url}</span>
                    <span class="text-xs opacity-60 flex-shrink-0">{"🔗"}</span>
                </a>
            </div>

            // Progress section or error message
            {if let Some(error) = &item.error_message {
                html! {
                    <div class="mt-auto">
                        <div class="p-1 bg-red-50 dark:bg-red-900/20 rounded border border-red-200 dark:border-red-800">
                            <div class="text-xs text-red-700 dark:text-red-300 break-words">
                                {if error.len() > 80 {
                                    format!("{}...", &error[..77])
                                } else {
                                    error.clone()
                                }}
                            </div>
                            {if error.contains("Video metadata was extracted successfully") {
                                html! {
                                    <div class="mt-1 text-xs text-amber-600 dark:text-amber-400">
                                        {"💡 Metadata found, streams not accessible"}
                                    </div>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                    </div>
                }
            } else if let Some(progress) = &item.download_progress {
                let progress_percentage = if progress.progress > 1.0 {
                    progress.progress.min(100.0).max(0.0)
                } else {
                    (progress.progress * 100.0).min(100.0).max(0.0)
                };

                html! {
                    <div class="mt-auto">
                        <div class="flex justify-between text-xs text-gray-600 dark:text-gray-400 mb-1">
                            <span>{format!("{:.0}%", progress_percentage)}</span>
                            {if let Some(speed) = progress.speed_bytes_per_sec {
                                html! {
                                    <span>{format!("{:.1} MB/s", speed as f64 / 1024.0 / 1024.0)}</span>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                        <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-1.5">
                            <div
                                class="bg-blue-600 h-1.5 rounded-full transition-all duration-300"
                                style={format!("width: {}%", progress_percentage)}
                            ></div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}
        </div>
    }
}
