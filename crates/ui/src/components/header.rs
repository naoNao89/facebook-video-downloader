use yew::prelude::*;
use crate::components::ThemeToggle;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <header class="fixed top-0 left-0 right-0 z-50 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 transition-colors duration-200 shadow-sm">
            <div class="px-4 sm:px-6 lg:px-8">
                <div class="flex justify-between items-center h-16">
                    <div class="flex items-center">
                        <div class="flex-shrink-0">
                            <div class="flex items-center space-x-3">
                                // App icon/logo
                                <div class="w-8 h-8 bg-gradient-to-br from-facebook-500 to-facebook-600 rounded-lg flex items-center justify-center">
                                    <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 24 24">
                                        <path d="M14 2H6C4.9 2 4 2.9 4 4V20C4 21.1 4.9 22 6 22H18C19.1 22 20 21.1 20 20V8L14 2Z" />
                                        <path d="M14 2V8H20" />
                                        <path d="M16 13H8M16 17H8M10 9H9H8" />
                                    </svg>
                                </div>
                                <h1 class="text-xl font-bold text-gray-900 dark:text-white">
                                    {"Facebook Video Downloader"}
                                </h1>
                            </div>
                        </div>
                    </div>

                    <div class="flex items-center space-x-4">
                        // Theme toggle button
                        <ThemeToggle />

                        // Additional header actions can go here
                        <div class="hidden sm:flex items-center space-x-2">
                            <span class="text-sm text-gray-500 dark:text-gray-400">
                                {"v1.0.0"}
                            </span>
                        </div>
                    </div>
                </div>
            </div>
        </header>
    }
}
