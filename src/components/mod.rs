// Components converted to Leptos
pub mod sidebar;
pub mod notification;
pub mod theme_toggle;
// pub mod modal; // Complex closure issues - needs advanced Leptos patterns

// Components still being migrated
// pub mod header;
pub mod icons;
pub mod copy_button;
pub mod privacy_indicator;
pub mod thumbnail;
// pub mod compression;
// pub mod batch_queue;
// pub mod modal;

// Re-exports for converted components
pub use sidebar::Sidebar;
pub use notification::{NotificationItem, NotificationList, NotificationContainer, Notification, NotificationType};
pub use theme_toggle::{ThemeToggle, ThemeToggleSwitch};
// pub use modal::{Modal, ConfirmModal, ModalSize}; // Temporarily disabled
pub use icons::{PasteIcon, LoadingIcon, PlayIcon, GlobeIcon, LockIcon, UsersIcon, ImagePlaceholderIcon, CheckIcon, ClipboardIcon};
pub use copy_button::CopyButton;
pub use privacy_indicator::{PrivacyIndicator, CompactPrivacyIndicator};
pub use thumbnail::Thumbnail;
