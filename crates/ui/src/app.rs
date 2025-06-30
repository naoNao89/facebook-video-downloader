use yew::prelude::*;
use yew_router::prelude::*;
use crate::components::{Header, Sidebar};
use crate::components::notification::NotificationContainer;
use crate::pages::*;
use crate::services::theme::ThemeProvider;
use crate::services::notification::NotificationProvider;
use crate::services::state::AppStateProvider;

/// Application routes
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/downloads")]
    Downloads,
    #[at("/batch")]
    Batch,
    #[at("/settings")]
    Settings,
    #[at("/about")]
    About,
    #[not_found]
    #[at("/404")]
    NotFound,
}

/// Route switching function
fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <HomePage /> },
        Route::Downloads => html! { <DownloadsPage /> },
        Route::Batch => html! { <BatchPage /> },
        Route::Settings => html! { <SettingsPage /> },
        Route::About => html! { <AboutPage /> },
        Route::NotFound => html! { <NotFoundPage /> },
    }
}

/// Main application component
#[function_component(App)]
pub fn app() -> Html {
    // Application startup effect
    use_effect_with((), |_| {
        log::info!("Application mounted successfully");
        || {}
    });

    html! {
        <AppStateProvider>
            <NotificationProvider>
                <ThemeProvider>
                    <BrowserRouter>
                        <div class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-200">
                            // Application header
                            <Header />

                            // Main content area
                            <div class="flex h-screen pt-16"> // pt-16 to account for fixed header
                                // Sidebar navigation
                                <Sidebar />

                                // Main content
                                <main class="flex-1 overflow-y-auto bg-white dark:bg-gray-800 transition-colors duration-200">
                                    <div class="container mx-auto px-4 py-6 max-w-7xl">
                                        <Switch<Route> render={switch} />
                                    </div>
                                </main>
                            </div>

                            // Global notification container
                            <NotificationContainer />
                        </div>
                    </BrowserRouter>
                </ThemeProvider>
            </NotificationProvider>
        </AppStateProvider>
    }
}

/// 404 Not Found page
#[function_component(NotFoundPage)]
pub fn not_found_page() -> Html {
    html! {
        <div class="flex flex-col items-center justify-center min-h-96 text-center">
            <div class="mb-8">
                <svg class="w-24 h-24 text-gray-400 dark:text-gray-600 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 12h6m-6-4h6m2 5.291A7.962 7.962 0 0112 15c-2.34 0-4.47-.881-6.08-2.33"></path>
                </svg>
                <h1 class="text-6xl font-bold text-gray-900 dark:text-white mb-2">{"404"}</h1>
                <h2 class="text-2xl font-semibold text-gray-700 dark:text-gray-300 mb-4">{"Page Not Found"}</h2>
                <p class="text-gray-600 dark:text-gray-400 mb-8 max-w-md">
                    {"The page you're looking for doesn't exist. It might have been moved, deleted, or you entered the wrong URL."}
                </p>
            </div>
            
            <div class="space-x-4">
                <Link<Route> to={Route::Home} classes="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-facebook-600 hover:bg-facebook-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-facebook-500 transition-colors duration-200">
                    <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"></path>
                    </svg>
                    {"Go Home"}
                </Link<Route>>
                
                <button 
                    onclick={Callback::from(|_| {
                        web_sys::window().unwrap().history().unwrap().back().unwrap();
                    })}
                    class="inline-flex items-center px-6 py-3 border border-gray-300 dark:border-gray-600 text-base font-medium rounded-md text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-facebook-500 transition-colors duration-200"
                >
                    <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"></path>
                    </svg>
                    {"Go Back"}
                </button>
            </div>
        </div>
    }
}
