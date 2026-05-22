use super::helpers::*;
use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_mpris(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // MPRIS media control
        "mpris.list" => Action::MprisList,
        "mpris.get" => Action::MprisGet {
            player: optional_non_empty_string(raw, "player")?,
        },
        "mpris.control" => Action::MprisControl {
            player: optional_non_empty_string(raw, "player")?,
            action: required_non_empty_string(raw, "action")?,
        },
        _ => anyhow::bail!("unknown mpris type: {type_str}"),
    })
}
