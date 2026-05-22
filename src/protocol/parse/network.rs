use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_network(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Network
        "network.status" => Action::NetworkStatus,
        "network.interfaces" => Action::NetworkInterfaces,
        "network.wifi.scan" => Action::NetworkWifiScan,
        "network.wifi.connect" => Action::NetworkWifiConnect {
            ssid: raw["ssid"].as_str().unwrap_or("").into(),
            password: raw["password"].as_str().map(String::from),
        },
        _ => anyhow::bail!("unknown network type: {type_str}"),
    })
}
