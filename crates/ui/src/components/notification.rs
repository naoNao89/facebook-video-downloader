use yew::prelude::*;

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

#[derive(Properties, PartialEq)]
pub struct NotificationProps {
    pub notification: Notification,
    pub on_close: Callback<String>,
}

#[function_component(NotificationItem)]
pub fn notification_item(props: &NotificationProps) -> Html {
    let notification = &props.notification;
    let on_close = props.on_close.clone();
    let id = notification.id.clone();

    let close_handler = {
        let on_close = on_close.clone();
        let id = id.clone();
        Callback::from(move |_| {
            on_close.emit(id.clone());
        })
    };

    let type_class = match notification.notification_type {
        NotificationType::Success => "bg-green-500",
        NotificationType::Error => "bg-red-500",
        NotificationType::Warning => "bg-yellow-500",
        NotificationType::Info => "bg-blue-500",
    };

    html! {
        <div class={format!("flex items-center justify-between p-4 mb-2 text-white rounded-lg shadow-lg {}", type_class)}>
            <span class="flex-1">{&notification.message}</span>
            <button
                onclick={close_handler}
                class="ml-4 text-white hover:text-gray-200 focus:outline-none"
            >
                {"×"}
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct NotificationContainerProps {
    pub notifications: Vec<Notification>,
    pub on_close: Callback<String>,
}

#[function_component(NotificationList)]
pub fn notification_list(props: &NotificationContainerProps) -> Html {
    html! {
        <div class="fixed top-4 right-4 z-50 w-80">
            {for props.notifications.iter().map(|notification| {
                html! {
                    <NotificationItem
                        key={notification.id.clone()}
                        notification={notification.clone()}
                        on_close={props.on_close.clone()}
                    />
                }
            })}
        </div>
    }
}

// Wrapper component that connects to the notification service
#[function_component(NotificationContainer)]
pub fn notification_container() -> Html {
    use crate::services::notification::use_notifications;

    let notification_service = use_notifications();

    html! {
        <NotificationList
            notifications={notification_service.notifications}
            on_close={notification_service.remove_notification}
        />
    }
}
