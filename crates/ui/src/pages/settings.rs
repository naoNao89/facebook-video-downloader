use yew::prelude::*;

#[function_component(SettingsPage)]
pub fn settings_page() -> Html {
    let download_path = use_state(|| String::from("~/Downloads"));
    let auto_retry = use_state(|| true);
    let max_concurrent = use_state(|| 3);
    let default_quality = use_state(|| String::from("best"));
    let notifications_enabled = use_state(|| true);
    let auto_open_folder = use_state(|| false);

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

    let on_save_settings = Callback::from(|_| {
        log::info!("Saving settings");
        // TODO: Implement settings save functionality
    });

    let on_reset_settings = Callback::from(|_| {
        log::info!("Resetting settings to defaults");
        // TODO: Implement settings reset functionality
    });

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
