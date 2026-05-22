use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_notifications(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Notifications
        "notification.send" => Action::NotificationSend {
            app_name: raw["app_name"].as_str().unwrap_or("deskbrid").into(),
            title: raw["title"].as_str().unwrap_or("").into(),
            body: raw["body"].as_str().unwrap_or("").into(),
            urgency: raw["urgency"].as_str().unwrap_or("normal").into(),
        },
        "notification.close" => Action::NotificationClose {
            notification_id: raw["notification_id"].as_u64().unwrap_or(0) as u32,
        },
        _ => anyhow::bail!("unknown notifications type: {type_str}"),
    })
}
