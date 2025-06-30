use yew::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_path: String,
    pub default_quality: String,
    pub max_concurrent_downloads: u32,
    pub auto_retry: bool,
    pub notifications_enabled: bool,
    pub auto_open_folder: bool,
    pub theme: String,
    pub language: String,
    // pub ipv6_settings: IPv6Settings, // Removed - not currently used
}

// IPv6Settings struct removed - not currently used

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            download_path: "~/Downloads".to_string(),
            default_quality: "best".to_string(),
            max_concurrent_downloads: 3,
            auto_retry: true,
            notifications_enabled: true,
            auto_open_folder: false,
            theme: "light".to_string(),
            language: "en".to_string(),
            // ipv6_settings: IPv6Settings::default(), // Removed - not currently used
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DownloadState {
    pub id: String,
    pub url: String,
    pub title: String,
    pub status: String,
    pub progress: f32,
    pub file_size: Option<u64>,
    pub downloaded_size: u64,
    pub download_speed: Option<f32>,
    pub eta: Option<u32>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub settings: AppSettings,
    pub downloads: Vec<DownloadState>,
    pub update_settings: Callback<AppSettings>,
    pub add_download: Callback<DownloadState>,
    pub update_download: Callback<DownloadState>,
    pub remove_download: Callback<String>,
}

impl PartialEq for AppState {
    fn eq(&self, other: &Self) -> bool {
        self.settings == other.settings && self.downloads == other.downloads
    }
}

#[derive(Properties, PartialEq)]
pub struct AppStateProviderProps {
    pub children: Children,
}

#[function_component(AppStateProvider)]
pub fn app_state_provider(props: &AppStateProviderProps) -> Html {
    let settings = use_state(|| AppSettings::default());
    let downloads = use_state(|| Vec::<DownloadState>::new());

    // Load settings from local storage on mount
    use_effect_with((), {
        let _settings = settings.clone();
        move |_| {
            // TODO: Load settings from local storage or Tauri store
            log::info!("Loading app settings...");
            || {}
        }
    });

    let update_settings = {
        let settings = settings.clone();
        Callback::from(move |new_settings: AppSettings| {
            settings.set(new_settings.clone());
            // TODO: Save settings to local storage or Tauri store
            log::info!("Settings updated: {:?}", new_settings);
        })
    };

    let add_download = {
        let downloads = downloads.clone();
        Callback::from(move |download: DownloadState| {
            let mut current = (*downloads).clone();
            current.push(download);
            downloads.set(current);
        })
    };

    let update_download = {
        let downloads = downloads.clone();
        Callback::from(move |updated_download: DownloadState| {
            let mut current = (*downloads).clone();
            if let Some(index) = current.iter().position(|d| d.id == updated_download.id) {
                current[index] = updated_download;
                downloads.set(current);
            }
        })
    };

    let remove_download = {
        let downloads = downloads.clone();
        Callback::from(move |id: String| {
            let mut current = (*downloads).clone();
            current.retain(|d| d.id != id);
            downloads.set(current);
        })
    };

    let state = AppState {
        settings: (*settings).clone(),
        downloads: (*downloads).clone(),
        update_settings,
        add_download,
        update_download,
        remove_download,
    };

    html! {
        <ContextProvider<AppState> context={state}>
            {props.children.clone()}
        </ContextProvider<AppState>>
    }
}

#[hook]
pub fn use_app_state() -> AppState {
    use_context::<AppState>().expect("App state context not found")
}
