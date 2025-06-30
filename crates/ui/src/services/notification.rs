use yew::prelude::*;
use crate::components::notification::{Notification, NotificationType};

#[derive(Clone)]
pub struct NotificationService {
    pub notifications: Vec<Notification>,
    pub add_notification: Callback<Notification>,
    pub remove_notification: Callback<String>,
    pub clear_all: Callback<()>,
}

impl PartialEq for NotificationService {
    fn eq(&self, other: &Self) -> bool {
        self.notifications == other.notifications
    }
}

#[derive(Properties, PartialEq)]
pub struct NotificationProviderProps {
    pub children: Children,
}

#[function_component(NotificationProvider)]
pub fn notification_provider(props: &NotificationProviderProps) -> Html {
    let notifications = use_state(|| Vec::<Notification>::new());

    let add_notification = {
        let notifications = notifications.clone();
        Callback::from(move |notification: Notification| {
            let mut current = (*notifications).clone();
            current.push(notification.clone());
            notifications.set(current);

            // Auto-remove notification after duration if specified
            if let Some(duration) = notification.duration {
                let notifications = notifications.clone();
                let id = notification.id.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    gloo::timers::future::TimeoutFuture::new(duration * 1000).await;
                    let mut current = (*notifications).clone();
                    current.retain(|n| n.id != id);
                    notifications.set(current);
                });
            }
        })
    };

    let remove_notification = {
        let notifications = notifications.clone();
        Callback::from(move |id: String| {
            let mut current = (*notifications).clone();
            current.retain(|n| n.id != id);
            notifications.set(current);
        })
    };

    let clear_all = {
        let notifications = notifications.clone();
        Callback::from(move |_| {
            notifications.set(Vec::new());
        })
    };

    let service = NotificationService {
        notifications: (*notifications).clone(),
        add_notification,
        remove_notification,
        clear_all,
    };

    html! {
        <ContextProvider<NotificationService> context={service}>
            {props.children.clone()}
        </ContextProvider<NotificationService>>
    }
}

#[hook]
pub fn use_notifications() -> NotificationService {
    use_context::<NotificationService>().expect("Notification service not found")
}

// Helper functions for common notification types
pub fn show_success(message: String) -> Notification {
    Notification {
        id: uuid::Uuid::new_v4().to_string(),
        message,
        notification_type: NotificationType::Success,
        duration: Some(5),
    }
}

pub fn show_error(message: String) -> Notification {
    Notification {
        id: uuid::Uuid::new_v4().to_string(),
        message,
        notification_type: NotificationType::Error,
        duration: None, // Errors persist until manually closed
    }
}

pub fn show_warning(message: String) -> Notification {
    Notification {
        id: uuid::Uuid::new_v4().to_string(),
        message,
        notification_type: NotificationType::Warning,
        duration: Some(8),
    }
}

pub fn show_info(message: String) -> Notification {
    Notification {
        id: uuid::Uuid::new_v4().to_string(),
        message,
        notification_type: NotificationType::Info,
        duration: Some(5),
    }
}
