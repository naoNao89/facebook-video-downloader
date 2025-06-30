use leptos::*;
use leptos::prelude::*;

#[derive(Clone, PartialEq)]
pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Clone, PartialEq)]
pub struct Notification {
    pub id: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub duration: Option<u32>, // Duration in seconds, None for persistent
}

#[component]
pub fn NotificationItem(
    notification: Notification,
    on_close: WriteSignal<Option<String>>,
) -> impl IntoView {
    let id = notification.id.clone();

    let close_handler = move |_| {
        on_close.set(Some(id.clone()));
    };

    let type_class = match notification.notification_type {
        NotificationType::Success => "bg-green-500",
        NotificationType::Error => "bg-red-500",
        NotificationType::Warning => "bg-yellow-500",
        NotificationType::Info => "bg-blue-500",
    };

    view! {
        <div class=format!("flex items-center justify-between p-4 mb-2 text-white rounded-lg shadow-lg {}", type_class)>
            <span class="flex-1">{notification.message}</span>
            <button
                on:click=close_handler
                class="ml-4 text-white hover:text-gray-200 focus:outline-none"
            >
                "×"
            </button>
        </div>
    }
}

#[component]
pub fn NotificationList(
    notifications: ReadSignal<Vec<Notification>>,
    on_close: WriteSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="fixed top-4 right-4 z-50 w-80">
            <For
                each=move || notifications.get()
                key=|notification| notification.id.clone()
                children=move |notification| {
                    view! {
                        <NotificationItem
                            notification=notification
                            on_close=on_close
                        />
                    }
                }
            />
        </div>
    }
}

// Wrapper component that connects to the notification service
#[component]
pub fn NotificationContainer() -> impl IntoView {
    // For now, create a simple local state until we convert the notification service
    let (notifications, set_notifications) = signal(Vec::<Notification>::new());
    let (close_id, set_close_id) = signal(None::<String>);

    // Handle notification removal
    Effect::new(move |_| {
        if let Some(id) = close_id.get() {
            set_notifications.update(|notifications| {
                notifications.retain(|n| n.id != id);
            });
            set_close_id.set(None);
        }
    });

    view! {
        <NotificationList
            notifications=notifications
            on_close=set_close_id
        />
    }
}
