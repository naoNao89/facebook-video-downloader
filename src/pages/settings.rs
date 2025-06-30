use leptos::*;
use leptos::prelude::*;
use crate::components::ThemeToggleSwitch;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let app_state = use_app_state();
    let settings = &app_state.settings;

    let download_path = use_state(|| settings.download_path.clone());
    let auto_retry = use_state(|| settings.auto_retry);
    let max_concurrent = use_state(|| settings.max_concurrent_downloads as i32);
    let default_quality = use_state(|| settings.default_quality.clone());
    let notifications_enabled = use_state(|| settings.notifications_enabled);
    let auto_open_folder = use_state(|| settings.auto_open_folder);

    let on_download_path_change = {
        let download_path = download_path.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            download_path.set(input.value());
        })
    };

    let on_browse_folder = Callback::from(|_| {
        log::info!("Opening folder browser");
        // TODO: Implement folder browser using Tauri dialog
    });

    let on_auto_retry_change = {
        let auto_retry = auto_retry.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            auto_retry.set(input.checked());
        })
    };

    let on_max_concurrent_change = {
        let max_concurrent = max_concurrent.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<i32>() {
                max_concurrent.set(value.max(1).min(10));
            }
        })
    };

    let on_quality_change = {
        let default_quality = default_quality.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            default_quality.set(input.value());
        })
    };

    let on_notifications_change = {
        let notifications_enabled = notifications_enabled.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            notifications_enabled.set(input.checked());
        })
    };

    let on_auto_open_change = {
        let auto_open_folder = auto_open_folder.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            auto_open_folder.set(input.checked());
        })
    };

    // IPv6 settings callback removed - not currently used

    let on_save_settings = {
        let app_state = app_state.clone();
        let download_path = download_path.clone();
        let auto_retry = auto_retry.clone();
        let max_concurrent = max_concurrent.clone();
        let default_quality = default_quality.clone();
        let notifications_enabled = notifications_enabled.clone();
        let auto_open_folder = auto_open_folder.clone();
        Callback::from(move |_| {
            let mut new_settings = app_state.settings.clone();
            new_settings.download_path = (*download_path).clone();
            new_settings.auto_retry = *auto_retry;
            new_settings.max_concurrent_downloads = (*max_concurrent).max(1).min(10) as u32;
            new_settings.default_quality = (*default_quality).clone();
            new_settings.notifications_enabled = *notifications_enabled;
            new_settings.auto_open_folder = *auto_open_folder;

            app_state.update_settings.emit(new_settings);
            log::info!("Settings saved successfully");
        })
    };

    let on_reset_settings = {
        let app_state = app_state.clone();
        Callback::from(move |_| {
            app_state.update_settings.emit(crate::services::state::AppSettings::default());
            log::info!("Settings reset to defaults");
        })
    };

    html! {
        <div class="p-6">
            <div class="max-w-4xl mx-auto">
                <div class="mb-6">
                    <h1 class="text-2xl font-bold text-gray-800 mb-2">{"Settings"}</h1>
                    <p class="text-gray-600">{"Configure your download preferences and application settings."}</p>
                </div>

                <div class="space-y-6">
                    // Download Settings
                    <div class="bg-white rounded-lg shadow-md p-6">
                        <h2 class="text-lg font-semibold text-gray-800 mb-4">{"Download Settings"}</h2>
                        
                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"Default Download Location"}
                                </label>
                                <div class="flex">
                                    <input
                                        type="text"
                                        class="flex-1 p-2 border border-gray-300 rounded-l-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                        value={(*download_path).clone()}
                                        onchange={on_download_path_change}
                                    />
                                    <button 
                                        class="px-4 py-2 bg-gray-500 text-white rounded-r-lg hover:bg-gray-600"
                                        onclick={on_browse_folder}
                                    >
                                        {"Browse"}
                                    </button>
                                </div>
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"Default Video Quality"}
                                </label>
                                <select 
                                    class="w-full p-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    value={(*default_quality).clone()}
                                    onchange={on_quality_change}
                                >
                                    <option value="best">{"Best Available"}</option>
                                    <option value="1080p">{"1080p"}</option>
                                    <option value="720p">{"720p"}</option>
                                    <option value="480p">{"480p"}</option>
                                    <option value="360p">{"360p"}</option>
                                </select>
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"Maximum Concurrent Downloads"}
                                </label>
                                <input
                                    type="number"
                                    min="1"
                                    max="10"
                                    class="w-full p-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    value={(*max_concurrent).to_string()}
                                    onchange={on_max_concurrent_change}
                                />
                                <p class="text-xs text-gray-500 mt-1">{"Recommended: 1-5 downloads"}</p>
                            </div>
                        </div>
                    </div>

                    // Behavior Settings
                    <div class="bg-white rounded-lg shadow-md p-6">
                        <h2 class="text-lg font-semibold text-gray-800 mb-4">{"Behavior Settings"}</h2>
                        
                        <div class="space-y-4">
                            <label class="flex items-center">
                                <input 
                                    type="checkbox" 
                                    class="mr-3"
                                    checked={*auto_retry}
                                    onchange={on_auto_retry_change}
                                />
                                <div>
                                    <span class="text-sm font-medium text-gray-700">{"Auto-retry failed downloads"}</span>
                                    <p class="text-xs text-gray-500">{"Automatically retry downloads that fail due to network issues"}</p>
                                </div>
                            </label>

                            <label class="flex items-center">
                                <input 
                                    type="checkbox" 
                                    class="mr-3"
                                    checked={*notifications_enabled}
                                    onchange={on_notifications_change}
                                />
                                <div>
                                    <span class="text-sm font-medium text-gray-700">{"Enable notifications"}</span>
                                    <p class="text-xs text-gray-500">{"Show desktop notifications when downloads complete"}</p>
                                </div>
                            </label>

                            <label class="flex items-center">
                                <input 
                                    type="checkbox" 
                                    class="mr-3"
                                    checked={*auto_open_folder}
                                    onchange={on_auto_open_change}
                                />
                                <div>
                                    <span class="text-sm font-medium text-gray-700">{"Auto-open download folder"}</span>
                                    <p class="text-xs text-gray-500">{"Automatically open the download folder when a download completes"}</p>
                                </div>
                            </label>
                        </div>
                    </div>

                    // Application Settings
                    <div class="bg-white rounded-lg shadow-md p-6">
                        <h2 class="text-lg font-semibold text-gray-800 mb-4">{"Application Settings"}</h2>

                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"Theme"}
                                </label>
                                <select class="w-full p-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent">
                                    <option value="light">{"Light"}</option>
                                    <option value="dark">{"Dark"}</option>
                                    <option value="system">{"System Default"}</option>
                                </select>
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"Language"}
                                </label>
                                <select class="w-full p-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent">
                                    <option value="en">{"English"}</option>
                                    <option value="es">{"Español"}</option>
                                    <option value="fr">{"Français"}</option>
                                    <option value="de">{"Deutsch"}</option>
                                </select>
                            </div>
                        </div>
                    </div>

                    // IPv6 Anti-Blocking Settings removed - not currently used

                    // Action Buttons
                    <div class="flex justify-between">
                        <button 
                            class="px-6 py-2 bg-gray-500 text-white rounded-lg hover:bg-gray-600"
                            onclick={on_reset_settings}
                        >
                            {"Reset to Defaults"}
                        </button>
                        <button 
                            class="px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
                            onclick={on_save_settings}
                        >
                            {"Save Settings"}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
