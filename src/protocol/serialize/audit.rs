use super::Action;
use serde_json::json;

pub(super) fn serialize_audit(action: &Action, id: &str) -> serde_json::Value {
    match action {
        // Audit
        Action::AuditLog {
            limit,
            action_type,
            status,
        } => {
            let mut obj = json!({"type": "audit.log", "id": id});
            if let Some(limit) = limit {
                obj["limit"] = json!(limit);
            }
            if let Some(action_type) = action_type {
                obj["action_type"] = json!(action_type);
            }
            if let Some(status) = status {
                obj["status"] = json!(status);
            }
            obj
        }
        Action::AuditClear => json!({"type": "audit.clear", "id": id}),

        // Notifications
        Action::NotificationSend {
            app_name,
            title,
            body,
            urgency,
        } => {
            json!({"type": "notification.send", "id": id, "app_name": app_name, "title": title, "body": body, "urgency": urgency})
        }
        Action::NotificationClose { notification_id } => {
            json!({"type": "notification.close", "id": id, "notification_id": notification_id})
        }
        _ => unreachable!("not a audit action"),
    }
}
