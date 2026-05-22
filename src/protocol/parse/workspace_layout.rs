use super::helpers::*;
use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_workspace_layout(
    raw: &Value,
    _id: &str,
    type_str: &str,
) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Workspaces
        "workspaces.list" => Action::WorkspacesList,
        "workspaces.switch" => {
            Action::WorkspaceSwitch(raw["workspace_id"].as_u64().unwrap_or(0) as u32)
        }
        "workspaces.move_window" => Action::WorkspaceMoveWindow {
            window_id: required_non_empty_string(raw, "window_id")?,
            workspace_id: raw["workspace_id"].as_u64().unwrap_or(0) as u32,
            follow: raw["follow"].as_bool().unwrap_or(false),
        },

        // Layout profiles
        "layout_profiles.list" => Action::LayoutProfilesList,
        "layout_profiles.get" => Action::LayoutProfileGet {
            name: required_non_empty_string(raw, "name")?,
        },
        "layout_profiles.save" => Action::LayoutProfileSave {
            name: required_non_empty_string(raw, "name")?,
            overwrite: raw["overwrite"].as_bool().unwrap_or(false),
        },
        "layout_profiles.delete" => Action::LayoutProfileDelete {
            name: required_non_empty_string(raw, "name")?,
        },
        "layout_profiles.restore" => Action::LayoutProfileRestore {
            name: required_non_empty_string(raw, "name")?,
        },
        _ => anyhow::bail!("unknown workspace_layout type: {type_str}"),
    })
}
