// Pages being migrated to Leptos
pub mod home;
// pub mod downloads; // Complex - needs component dependencies
// pub mod batch; // Complex - needs component dependencies
// pub mod settings; // Complex - needs state service
pub mod about;

// Re-exports for converted pages
pub use home::HomePage;
// pub use downloads::DownloadsPage;
// pub use batch::BatchPage;
// pub use settings::SettingsPage;
pub use about::AboutPage;
