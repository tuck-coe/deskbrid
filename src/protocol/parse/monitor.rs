use super::helpers::*;
use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_monitor(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Monitor
        "monitor.list" => Action::MonitorList,
        "monitor.set_primary" => Action::MonitorSetPrimary {
            output: required_non_empty_string(raw, "output")?,
        },
        "monitor.set_resolution" => {
            let refresh_rate = match optional_positive_f64(raw, "refresh_rate")? {
                Some(refresh_rate) => Some(refresh_rate),
                None => optional_positive_f64(raw, "refresh")?,
            };
            Action::MonitorSetResolution {
                output: required_non_empty_string(raw, "output")?,
                width: required_positive_u32(raw, "width")?,
                height: required_positive_u32(raw, "height")?,
                refresh_rate,
            }
        }
        "monitor.set_scale" => Action::MonitorSetScale {
            output: required_non_empty_string(raw, "output")?,
            scale: required_positive_f64(raw, "scale")?,
        },
        "monitor.set_rotation" => Action::MonitorSetRotation {
            output: required_non_empty_string(raw, "output")?,
            rotation: required_rotation(raw, "rotation")?,
        },
        "monitor.enable" => Action::MonitorEnable {
            output: required_non_empty_string(raw, "output")?,
        },
        "monitor.disable" => Action::MonitorDisable {
            output: required_non_empty_string(raw, "output")?,
        },
        _ => anyhow::bail!("unknown monitor type: {type_str}"),
    })
}
