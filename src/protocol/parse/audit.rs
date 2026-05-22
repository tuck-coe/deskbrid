use super::helpers::*;
use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_audit(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Audit
        "audit.log" => Action::AuditLog {
            limit: raw["limit"].as_u64().map(|value| value as usize),
            action_type: optional_non_empty_string(raw, "action_type")?,
            status: optional_non_empty_string(raw, "status")?,
        },
        "audit.clear" => Action::AuditClear,
        _ => anyhow::bail!("unknown audit type: {type_str}"),
    })
}
