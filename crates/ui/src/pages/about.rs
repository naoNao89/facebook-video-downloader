use yew::prelude::*;

#[function_component(AboutPage)]
pub fn about_page() -> Html {
    let app_version = env!("CARGO_PKG_VERSION");
    
    html! {
        <div class="p-6">
            <div class="max-w-4xl mx-auto">
                <div class="text-center mb-8">
                    <div class="text-6xl mb-4">{"📱"}</div>
                    <h1 class="text-3xl font-bold text-gray-800 mb-2">{"Facebook Video Downloader"}</h1>
                    <p class="text-lg text-gray-600">{"Version "}{app_version}</p>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
                    <div class="bg-white rounded-lg shadow-md p-6">
                        <h2 class="text-xl font-semibold text-gray-800 mb-4">{"About"}</h2>
                        <p class="text-gray-600 mb-4">
                            {"Facebook Video Downloader is a powerful desktop application built with Tauri, Yew, and Rust. "}
                            {"It allows you to easily download videos from Facebook with high quality and fast speeds."}
                        </p>
                        <p class="text-gray-600">
                            {"The application is designed to be user-friendly, secure, and efficient, providing a seamless "}
                            {"experience for downloading your favorite Facebook videos."}
                        </p>
                    </div>

                    <div class="bg-white rounded-lg shadow-md p-6">
                        <h2 class="text-xl font-semibold text-gray-800 mb-4">{"Features"}</h2>
                        <ul class="text-gray-600 space-y-2">
                            <li class="flex items-center">
                                <span class="text-green-500 mr-2">{"✓"}</span>
                                {"High-quality video downloads"}
                            </li>
                            <li class="flex items-center">
                                <span class="text-green-500 mr-2">{"✓"}</span>
                                {"Batch download support"}
                            </li>
                            <li class="flex items-center">
                                <span class="text-green-500 mr-2">{"✓"}</span>
                                {"Multiple quality options"}
                            </li>
                            <li class="flex items-center">
                                <span class="text-green-500 mr-2">{"✓"}</span>
                                {"Progress tracking"}
                            </li>
                            <li class="flex items-center">
                                <span class="text-green-500 mr-2">{"✓"}</span>
                                {"Cross-platform support"}
                            </li>
                            <li class="flex items-center">
                                <span class="text-green-500 mr-2">{"✓"}</span>
                                {"Modern, intuitive interface"}
                            </li>
                        </ul>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                    <h2 class="text-xl font-semibold text-gray-800 mb-4">{"Technology Stack"}</h2>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                        <div class="text-center">
                            <div class="text-2xl mb-2">{"🦀"}</div>
                            <h3 class="font-semibold text-gray-700">{"Rust"}</h3>
                            <p class="text-sm text-gray-500">{"Backend Logic"}</p>
                        </div>
                        <div class="text-center">
                            <div class="text-2xl mb-2">{"⚡"}</div>
                            <h3 class="font-semibold text-gray-700">{"Tauri"}</h3>
                            <p class="text-sm text-gray-500">{"Desktop Framework"}</p>
                        </div>
                        <div class="text-center">
                            <div class="text-2xl mb-2">{"🌐"}</div>
                            <h3 class="font-semibold text-gray-700">{"Yew"}</h3>
                            <p class="text-sm text-gray-500">{"Frontend Framework"}</p>
                        </div>
                        <div class="text-center">
                            <div class="text-2xl mb-2">{"🎨"}</div>
                            <h3 class="font-semibold text-gray-700">{"Tailwind CSS"}</h3>
                            <p class="text-sm text-gray-500">{"Styling"}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                    <h2 class="text-xl font-semibold text-gray-800 mb-4">{"System Information"}</h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <h3 class="font-semibold text-gray-700 mb-2">{"Application"}</h3>
                            <ul class="text-sm text-gray-600 space-y-1">
                                <li>{"Version: "}{app_version}</li>
                                <li>{"Build: Release"}</li>
                                <li>{"Platform: Desktop"}</li>
                            </ul>
                        </div>
                        <div>
                            <h3 class="font-semibold text-gray-700 mb-2">{"Dependencies"}</h3>
                            <ul class="text-sm text-gray-600 space-y-1">
                                <li>{"Tauri: 2.x"}</li>
                                <li>{"Yew: 0.21"}</li>
                                <li>{"Rust: 2024 Edition"}</li>
                            </ul>
                        </div>
                    </div>
                </div>

                <div class="bg-blue-50 border border-blue-200 rounded-lg p-6 mb-6">
                    <h2 class="text-xl font-semibold text-blue-800 mb-4">{"Legal Notice"}</h2>
                    <p class="text-sm text-blue-700 mb-2">
                        {"This application is for personal use only. Please respect copyright laws and Facebook's terms of service."}
                    </p>
                    <p class="text-sm text-blue-700">
                        {"Only download videos that you have permission to download or that are in the public domain."}
                    </p>
                </div>

                <div class="text-center">
                    <div class="flex justify-center space-x-4">
                        <button class="px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600">
                            {"Check for Updates"}
                        </button>
                        <button class="px-6 py-2 bg-gray-500 text-white rounded-lg hover:bg-gray-600">
                            {"View License"}
                        </button>
                        <button class="px-6 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600">
                            {"Report Issue"}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
