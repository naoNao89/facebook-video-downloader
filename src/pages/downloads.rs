use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct DownloadItem {
    pub id: String,
    pub url: String,
    pub title: String,
    pub status: DownloadStatus,
    pub progress: f32,
    pub file_size: Option<u64>,
    pub downloaded_size: u64,
    pub download_speed: Option<f32>,
    pub eta: Option<u32>,
}

#[derive(Clone, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Paused,
}

impl DownloadStatus {
    pub fn to_string(&self) -> &'static str {
        match self {
            DownloadStatus::Pending => "Pending",
            DownloadStatus::Downloading => "Downloading",
            DownloadStatus::Completed => "Completed",
            DownloadStatus::Failed => "Failed",
            DownloadStatus::Paused => "Paused",
        }
    }

    pub fn get_color_class(&self) -> &'static str {
        match self {
            DownloadStatus::Pending => "text-yellow-600",
            DownloadStatus::Downloading => "text-blue-600",
            DownloadStatus::Completed => "text-green-600",
            DownloadStatus::Failed => "text-red-600",
            DownloadStatus::Paused => "text-gray-600",
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct DownloadItemProps {
    pub item: DownloadItem,
    pub on_pause: Callback<String>,
    pub on_resume: Callback<String>,
    pub on_cancel: Callback<String>,
    pub on_retry: Callback<String>,
}

#[function_component(DownloadItemComponent)]
pub fn download_item_component(props: &DownloadItemProps) -> Html {
    let item = &props.item;
    
    let format_size = |size: u64| -> String {
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    };

    let format_speed = |speed: f32| -> String {
        if speed < 1024.0 {
            format!("{:.1} B/s", speed)
        } else if speed < 1024.0 * 1024.0 {
            format!("{:.1} KB/s", speed / 1024.0)
        } else {
            format!("{:.1} MB/s", speed / (1024.0 * 1024.0))
        }
    };

    let format_time = |seconds: u32| -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    };

    html! {
        <div class="bg-white rounded-lg shadow-md p-4 mb-4">
            <div class="flex items-center justify-between mb-2">
                <h3 class="text-lg font-semibold text-gray-800 truncate flex-1 mr-4">
                    {&item.title}
                </h3>
                <span class={format!("text-sm font-medium {}", item.status.get_color_class())}>
                    {item.status.to_string()}
                </span>
            </div>
            
            <div class="text-sm text-gray-600 mb-2 truncate">
                {&item.url}
            </div>
            
            if item.status == DownloadStatus::Downloading || item.status == DownloadStatus::Completed {
                <div class="mb-2">
                    <div class="flex justify-between text-sm text-gray-600 mb-1">
                        <span>{format!("{}%", (item.progress * 100.0) as u32)}</span>
                        <span>
                            {format_size(item.downloaded_size)}
                            if let Some(total) = item.file_size {
                                {format!(" / {}", format_size(total))}
                            }
                        </span>
                    </div>
                    <div class="w-full bg-gray-200 rounded-full h-2">
                        <div 
                            class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                            style={format!("width: {}%", item.progress * 100.0)}
                        ></div>
                    </div>
                </div>
            }
            
            if item.status == DownloadStatus::Downloading {
                <div class="flex justify-between text-sm text-gray-600 mb-2">
                    if let Some(speed) = item.download_speed {
                        <span>{"Speed: "}{format_speed(speed)}</span>
                    }
                    if let Some(eta) = item.eta {
                        <span>{"ETA: "}{format_time(eta)}</span>
                    }
                </div>
            }
            
            <div class="flex space-x-2">
                {match item.status {
                    DownloadStatus::Downloading => html! {
                        <>
                            <button 
                                class="px-3 py-1 bg-yellow-500 text-white rounded hover:bg-yellow-600 text-sm"
                                onclick={
                                    let on_pause = props.on_pause.clone();
                                    let id = item.id.clone();
                                    Callback::from(move |_| on_pause.emit(id.clone()))
                                }
                            >
                                {"Pause"}
                            </button>
                            <button 
                                class="px-3 py-1 bg-red-500 text-white rounded hover:bg-red-600 text-sm"
                                onclick={
                                    let on_cancel = props.on_cancel.clone();
                                    let id = item.id.clone();
                                    Callback::from(move |_| on_cancel.emit(id.clone()))
                                }
                            >
                                {"Cancel"}
                            </button>
                        </>
                    },
                    DownloadStatus::Paused => html! {
                        <>
                            <button 
                                class="px-3 py-1 bg-green-500 text-white rounded hover:bg-green-600 text-sm"
                                onclick={
                                    let on_resume = props.on_resume.clone();
                                    let id = item.id.clone();
                                    Callback::from(move |_| on_resume.emit(id.clone()))
                                }
                            >
                                {"Resume"}
                            </button>
                            <button 
                                class="px-3 py-1 bg-red-500 text-white rounded hover:bg-red-600 text-sm"
                                onclick={
                                    let on_cancel = props.on_cancel.clone();
                                    let id = item.id.clone();
                                    Callback::from(move |_| on_cancel.emit(id.clone()))
                                }
                            >
                                {"Cancel"}
                            </button>
                        </>
                    },
                    DownloadStatus::Failed => html! {
                        <button 
                            class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 text-sm"
                            onclick={
                                let on_retry = props.on_retry.clone();
                                let id = item.id.clone();
                                Callback::from(move |_| on_retry.emit(id.clone()))
                            }
                        >
                            {"Retry"}
                        </button>
                    },
                    _ => html! {}
                }}
            </div>
        </div>
    }
}

#[function_component(DownloadsPage)]
pub fn downloads_page() -> Html {
    let downloads = use_state(|| Vec::<DownloadItem>::new());

    let on_pause = Callback::from(|id: String| {
        log::info!("Pausing download: {}", id);
        // TODO: Implement pause functionality
    });

    let on_resume = Callback::from(|id: String| {
        log::info!("Resuming download: {}", id);
        // TODO: Implement resume functionality
    });

    let on_cancel = Callback::from(|id: String| {
        log::info!("Cancelling download: {}", id);
        // TODO: Implement cancel functionality
    });

    let on_retry = Callback::from(|id: String| {
        log::info!("Retrying download: {}", id);
        // TODO: Implement retry functionality
    });

    html! {
        <div class="p-6">
            <div class="flex items-center justify-between mb-6">
                <h1 class="text-2xl font-bold text-gray-800">{"Downloads"}</h1>
                <div class="flex space-x-2">
                    <button class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600">
                        {"Clear Completed"}
                    </button>
                    <button class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600">
                        {"Clear All"}
                    </button>
                </div>
            </div>

            if downloads.is_empty() {
                <div class="text-center py-12">
                    <div class="text-gray-400 text-6xl mb-4">{"📥"}</div>
                    <h2 class="text-xl font-semibold text-gray-600 mb-2">{"No downloads yet"}</h2>
                    <p class="text-gray-500">{"Start downloading videos from the Home page"}</p>
                </div>
            } else {
                <div class="space-y-4">
                    {for downloads.iter().map(|item| {
                        html! {
                            <DownloadItemComponent
                                key={item.id.clone()}
                                item={item.clone()}
                                on_pause={on_pause.clone()}
                                on_resume={on_resume.clone()}
                                on_cancel={on_cancel.clone()}
                                on_retry={on_retry.clone()}
                            />
                        }
                    })}
                </div>
            }
        </div>
    }
}
