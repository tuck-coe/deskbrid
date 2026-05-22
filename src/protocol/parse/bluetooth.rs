use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_bluetooth(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Bluetooth
        "bluetooth.list" => Action::BluetoothList,
        "bluetooth.scan" => Action::BluetoothScan {
            duration: raw["duration"].as_u64().map(|v| v as u32),
        },
        "bluetooth.scan_stop" => Action::BluetoothStopScan,
        "bluetooth.connect" => Action::BluetoothConnect {
            address: raw["address"].as_str().unwrap_or("").into(),
        },
        "bluetooth.disconnect" => Action::BluetoothDisconnect {
            address: raw["address"].as_str().unwrap_or("").into(),
        },
        "bluetooth.pair" => Action::BluetoothPair {
            address: raw["address"].as_str().unwrap_or("").into(),
        },
        "bluetooth.forget" => Action::BluetoothForget {
            address: raw["address"].as_str().unwrap_or("").into(),
        },
        _ => anyhow::bail!("unknown bluetooth type: {type_str}"),
    })
}
