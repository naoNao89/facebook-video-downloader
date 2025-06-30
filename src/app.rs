use leptos::*;
use leptos::prelude::*;
use leptos_router::*;
use leptos_router::components::{Router, Routes, Route, A};
use crate::pages::{AboutPage, HomePage};

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    // Application startup effect
    Effect::new(move |_| {
        log::info!("Application mounted successfully");
    });

    view! {
        <Router>
            <div class="min-h-screen bg-gray-50 transition-colors duration-200">
                // Application header
                <header class="fixed top-0 left-0 right-0 z-50 bg-white border-b border-gray-200 shadow-sm">
                    <div class="px-4 sm:px-6 lg:px-8">
                        <div class="flex justify-between items-center h-16">
                            <div class="flex items-center">
                                <h1 class="text-xl font-bold text-gray-900">
                                    "Facebook Video Downloader"
                                </h1>
                            </div>
                        </div>
                    </div>
                </header>

                // Main content
                <main class="pt-16 p-6">
                    <div class="container mx-auto max-w-7xl">
                        <Routes fallback=NotFoundPage>
                            <Route path=StaticSegment("") view=HomePage/>
                            <Route path=StaticSegment("about") view=AboutPage/>
                        </Routes>
                    </div>
                </main>
            </div>
        </Router>
    }
}



/// 404 Not Found page
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-96 text-center">
            <div class="mb-8">
                <svg class="w-24 h-24 text-gray-400 dark:text-gray-600 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 12h6m-6-4h6m2 5.291A7.962 7.962 0 0112 15c-2.34 0-4.47-.881-6.08-2.33"></path>
                </svg>
                <h1 class="text-6xl font-bold text-gray-900 dark:text-white mb-2">"404"</h1>
                <h2 class="text-2xl font-semibold text-gray-700 dark:text-gray-300 mb-4">"Page Not Found"</h2>
                <p class="text-gray-600 dark:text-gray-400 mb-8 max-w-md">
                    "The page you're looking for doesn't exist. It might have been moved, deleted, or you entered the wrong URL."
                </p>
            </div>

            <div class="space-x-4">
                <A href="/" attr:class="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-facebook-600 hover:bg-facebook-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-facebook-500 transition-colors duration-200">
                    <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"></path>
                    </svg>
                    "Go Home"
                </A>

                <button
                    on:click=move |_| {
                        if let Some(window) = web_sys::window() {
                            if let Some(history) = window.history().ok() {
                                let _ = history.back();
                            }
                        }
                    }
                    class="inline-flex items-center px-6 py-3 border border-gray-300 dark:border-gray-600 text-base font-medium rounded-md text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-facebook-500 transition-colors duration-200"
                >
                    <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"></path>
                    </svg>
                    "Go Back"
                </button>
            </div>
        </div>
    }
}
