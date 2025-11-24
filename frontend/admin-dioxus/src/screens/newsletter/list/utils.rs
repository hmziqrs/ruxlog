use ruxlog_shared::store::SubscriberStatus;

pub fn format_subscriber_status(status: SubscriberStatus) -> &'static str {
    match status {
        SubscriberStatus::Pending => "Pending",
        SubscriberStatus::Confirmed => "Confirmed",
        SubscriberStatus::Unsubscribed => "Unsubscribed",
    }
}

pub fn status_badge_class(status: SubscriberStatus) -> &'static str {
    match status {
        SubscriberStatus::Confirmed => "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
        SubscriberStatus::Pending => "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200",
        SubscriberStatus::Unsubscribed => "bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200",
    }
}
