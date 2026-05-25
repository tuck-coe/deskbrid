use super::*;
use crate::protocol;

pub(super) async fn network_status(
    backend: &KdeBackend,
) -> anyhow::Result<protocol::NetworkStatusInfo> {
    let out = backend
        .sh("nmcli", &["-t", "-f", "STATE", "general"])
        .await?;
    Ok(protocol::NetworkStatusInfo {
        online: out.trim().contains("connected"),
        net_type: String::new(),
    })
}

pub(super) async fn network_interfaces(
    backend: &KdeBackend,
) -> anyhow::Result<Vec<protocol::NetworkInterfaceInfo>> {
    let out = backend
        .sh(
            "nmcli",
            &["-t", "-f", "NAME,TYPE,DEVICE,STATE,IP4", "device", "status"],
        )
        .await?;
    let mut interfaces = Vec::new();
    for line in out.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            interfaces.push(protocol::NetworkInterfaceInfo {
                name: parts.first().unwrap_or(&"").to_string(),
                state: parts.get(3).unwrap_or(&"").to_string(),
                ipv4: parts
                    .get(4)
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty()),
                ipv6: None,
            });
        }
    }
    Ok(interfaces)
}

pub(super) async fn wifi_scan(
    backend: &KdeBackend,
) -> anyhow::Result<Vec<protocol::WifiNetworkInfo>> {
    let out = backend
        .sh(
            "nmcli",
            &["-t", "-f", "SSID,SIGNAL,SECURITY", "device", "wifi", "list"],
        )
        .await?;
    let mut networks = Vec::new();
    for line in out.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            networks.push(protocol::WifiNetworkInfo {
                ssid: parts[0].to_string(),
                strength: parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
                secured: !parts
                    .get(2)
                    .map(|s| s.is_empty() || s.contains("--"))
                    .unwrap_or(true),
                frequency: None,
            });
        }
    }
    Ok(networks)
}

pub(super) async fn wifi_connect(
    backend: &KdeBackend,
    ssid: &str,
    password: Option<&str>,
) -> anyhow::Result<()> {
    let ssid_escaped = ssid.replace('\\', "\\\\").replace('\'', "\\'");
    if let Some(pass) = password {
        backend
            .sh(
                "nmcli",
                &["device", "wifi", "connect", &ssid_escaped, "password", pass],
            )
            .await?;
    } else {
        backend
            .sh("nmcli", &["device", "wifi", "connect", &ssid_escaped])
            .await?;
    }
    Ok(())
}

// ── Bluetooth via BlueZ D-Bus (busctl) ──────────────────────────────

fn find_adapter_path(managed: &serde_json::Value) -> Option<String> {
    let data = managed.get("data")?.as_array()?;
    for entry in data {
        let pair = entry.get("data")?.as_array()?;
        if pair.len() < 2 { continue; }
        let path = pair[0].get("data")?.as_str()?;
        let ifaces = &pair[1];
        for iface_entry in ifaces.get("data")?.as_array()? {
            let iface_pair = iface_entry.get("data")?.as_array()?;
            if iface_pair.len() < 2 { continue; }
            if iface_pair[0].get("data")?.as_str() == Some("org.bluez.Adapter1") {
                return Some(path.to_string());
            }
        }
    }
    None
}

fn parse_device_props(props_val: &serde_json::Value) -> (String, String, bool, bool, Option<i32>) {
    let mut address = String::new();
    let mut name = "(unknown)".to_string();
    let mut paired = false;
    let mut connected = false;
    let mut rssi: Option<i32> = None;

    let props = match props_val.get("data") {
        Some(d) => d.as_array(),
        None => return (address, name, paired, connected, rssi),
    };
    let Some(props) = props else {
        return (address, name, paired, connected, rssi);
    };

    for prop_entry in props {
        let pair = match prop_entry.get("data") {
            Some(d) => d.as_array(),
            None => continue,
        };
        let Some(pair) = pair else { continue };
        if pair.len() < 2 { continue; }
        let prop_name = pair[0].get("data").and_then(|v| v.as_str()).unwrap_or("");
        let variant = &pair[1];
        // variant is {"type":"v","data":[{"type":"s","data":"..."}]}
        let inner = variant.get("data").and_then(|v| v.as_array()).and_then(|a| a.first());

        match prop_name {
            "Address" => {
                if let Some(v) = inner.and_then(|v| v.get("data")).and_then(|v| v.as_str()) {
                    address = v.to_string();
                }
            }
            "Name" => {
                if let Some(v) = inner.and_then(|v| v.get("data")).and_then(|v| v.as_str()) {
                    name = v.to_string();
                }
            }
            "Paired" => {
                paired = inner.and_then(|v| v.get("data")).and_then(|v| v.as_bool()).unwrap_or(false);
            }
            "Connected" => {
                connected = inner.and_then(|v| v.get("data")).and_then(|v| v.as_bool()).unwrap_or(false);
            }
            "RSSI" => {
                rssi = inner.and_then(|v| v.get("data")).and_then(|v| v.as_i64()).map(|v| v as i32);
            }
            _ => {}
        }
    }

    (address, name, paired, connected, rssi)
}

pub(super) async fn bluetooth_list(
    backend: &KdeBackend,
) -> anyhow::Result<Vec<protocol::BluetoothDeviceInfo>> {
    let raw = match backend.sh(
        "busctl",
        &["--json=short", "--system", "call", "org.bluez", "/",
          "org.freedesktop.DBus.ObjectManager", "GetManagedObjects"],
    ).await {
        Ok(out) => out,
        Err(_) => return Ok(Vec::new()), // BlueZ not available
    };
    let managed: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    let mut devices = Vec::new();
    let data = match managed.get("data") {
        Some(d) => d.as_array(),
        None => return Ok(devices),
    };
    let Some(entries) = data else { return Ok(devices) };

    for entry in entries {
        let pair = match entry.get("data") {
            Some(d) => d.as_array(),
            None => continue,
        };
        let Some(pair) = pair else { continue };
        if pair.len() < 2 { continue; }

        let ifaces = &pair[1];
        let iface_entries = match ifaces.get("data") {
            Some(d) => d.as_array(),
            None => continue,
        };
        let Some(iface_entries) = iface_entries else { continue };

        for iface_entry in iface_entries {
            let iface_pair = match iface_entry.get("data") {
                Some(d) => d.as_array(),
                None => continue,
            };
            let Some(iface_pair) = iface_pair else { continue };
            if iface_pair.len() < 2 { continue; }

            let iface_name = iface_pair[0].get("data").and_then(|v| v.as_str()).unwrap_or("");
            if iface_name != "org.bluez.Device1" {
                continue;
            }

            let (address, name, paired, connected, rssi) = parse_device_props(&iface_pair[1]);
            if address.is_empty() {
                continue;
            }
            devices.push(protocol::BluetoothDeviceInfo {
                address,
                name,
                paired,
                connected,
                rssi,
            });
        }
    }

    Ok(devices)
}

fn device_path(address: &str) -> String {
    format!("/org/bluez/hci0/dev_{}", address.replace(':', "_"))
}

pub(super) async fn bluetooth_scan(
    backend: &KdeBackend,
    _duration: Option<u32>,
) -> anyhow::Result<()> {
    // Find adapter path first
    let adapter = match find_adapter(backend).await {
        Some(a) => a,
        None => anyhow::bail!("no Bluetooth adapter found"),
    };
    backend.sh(
        "busctl",
        &["--system", "call", "org.bluez", &adapter,
          "org.bluez.Adapter1", "StartDiscovery"],
    ).await?;
    Ok(())
}

pub(super) async fn bluetooth_stop_scan(backend: &KdeBackend) -> anyhow::Result<()> {
    let adapter = match find_adapter(backend).await {
        Some(a) => a,
        None => return Ok(()), // no adapter, nothing to stop
    };
    let _ = backend.sh(
        "busctl",
        &["--system", "call", "org.bluez", &adapter,
          "org.bluez.Adapter1", "StopDiscovery"],
    ).await;
    Ok(())
}

pub(super) async fn bluetooth_connect(backend: &KdeBackend, address: &str) -> anyhow::Result<()> {
    let path = device_path(address);
    backend.sh(
        "busctl",
        &["--system", "call", "org.bluez", &path,
          "org.bluez.Device1", "Connect"],
    ).await?;
    Ok(())
}

pub(super) async fn bluetooth_disconnect(
    backend: &KdeBackend,
    address: &str,
) -> anyhow::Result<()> {
    let path = device_path(address);
    backend.sh(
        "busctl",
        &["--system", "call", "org.bluez", &path,
          "org.bluez.Device1", "Disconnect"],
    ).await?;
    Ok(())
}

async fn find_adapter(backend: &KdeBackend) -> Option<String> {
    let raw = backend.sh(
        "busctl",
        &["--json=short", "--system", "call", "org.bluez", "/",
          "org.freedesktop.DBus.ObjectManager", "GetManagedObjects"],
    ).await.ok()?;
    let managed: serde_json::Value = serde_json::from_str(&raw).ok()?;
    find_adapter_path(&managed)
}
