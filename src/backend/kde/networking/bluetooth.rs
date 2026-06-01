// Bluetooth via BlueZ D-Bus (busctl)

use crate::backend::kde::KdeBackend;
use crate::protocol;
use serde_json;

fn find_adapter_path(managed: &serde_json::Value) -> Option<String> {
    let data = managed.get("data")?.as_array()?;
    for entry in data {
        let pair = entry.get("data")?.as_array()?;
        if pair.len() < 2 {
            continue;
        }
        let path = pair[0].get("data")?.as_str()?;
        let ifaces = &pair[1];
        for iface_entry in ifaces.get("data")?.as_array()? {
            let iface_pair = iface_entry.get("data")?.as_array()?;
            if iface_pair.len() < 2 {
                continue;
            }
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
        if pair.len() < 2 {
            continue;
        }
        let prop_name = pair[0].get("data").and_then(|v| v.as_str()).unwrap_or("");
        let variant = &pair[1];
        // variant is {"type":"v","data":[{"type":"s","data":"..."}]}
        let inner = variant
            .get("data")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first());

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
                paired = inner
                    .and_then(|v| v.get("data"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
            }
            "Connected" => {
                connected = inner
                    .and_then(|v| v.get("data"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
            }
            "RSSI" => {
                rssi = inner
                    .and_then(|v| v.get("data"))
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);
            }
            _ => {}
        }
    }

    (address, name, paired, connected, rssi)
}

pub(crate) async fn bluetooth_list(
    backend: &KdeBackend,
) -> anyhow::Result<Vec<protocol::BluetoothDeviceInfo>> {
    let raw = match backend
        .sh(
            "busctl",
            &[
                "--json=short",
                "--system",
                "call",
                "org.bluez",
                "/",
                "org.freedesktop.DBus.ObjectManager",
                "GetManagedObjects",
            ],
        )
        .await
    {
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
    let Some(entries) = data else {
        return Ok(devices);
    };

    for entry in entries {
        let pair = match entry.get("data") {
            Some(d) => d.as_array(),
            None => continue,
        };
        let Some(pair) = pair else { continue };
        if pair.len() < 2 {
            continue;
        }

        let ifaces = &pair[1];
        let iface_entries = match ifaces.get("data") {
            Some(d) => d.as_array(),
            None => continue,
        };
        let Some(iface_entries) = iface_entries else {
            continue;
        };

        for iface_entry in iface_entries {
            let iface_pair = match iface_entry.get("data") {
                Some(d) => d.as_array(),
                None => continue,
            };
            let Some(iface_pair) = iface_pair else {
                continue;
            };
            if iface_pair.len() < 2 {
                continue;
            }

            let iface_name = iface_pair[0]
                .get("data")
                .and_then(|v| v.as_str())
                .unwrap_or("");
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

pub(crate) async fn bluetooth_scan(
    backend: &KdeBackend,
    _duration: Option<u32>,
) -> anyhow::Result<()> {
    // Find adapter path first
    let adapter = match find_adapter(backend).await {
        Some(a) => a,
        None => anyhow::bail!("no Bluetooth adapter found"),
    };
    backend
        .sh(
            "busctl",
            &[
                "--system",
                "call",
                "org.bluez",
                &adapter,
                "org.bluez.Adapter1",
                "StartDiscovery",
            ],
        )
        .await?;
    Ok(())
}

pub(crate) async fn bluetooth_stop_scan(backend: &KdeBackend) -> anyhow::Result<()> {
    let adapter = match find_adapter(backend).await {
        Some(a) => a,
        None => return Ok(()), // no adapter, nothing to stop
    };
    let _ = backend
        .sh(
            "busctl",
            &[
                "--system",
                "call",
                "org.bluez",
                &adapter,
                "org.bluez.Adapter1",
                "StopDiscovery",
            ],
        )
        .await;
    Ok(())
}

pub(crate) async fn bluetooth_connect(backend: &KdeBackend, address: &str) -> anyhow::Result<()> {
    let path = device_path(address);
    backend
        .sh(
            "busctl",
            &[
                "--system",
                "call",
                "org.bluez",
                &path,
                "org.bluez.Device1",
                "Connect",
            ],
        )
        .await?;
    Ok(())
}

pub(crate) async fn bluetooth_disconnect(
    backend: &KdeBackend,
    address: &str,
) -> anyhow::Result<()> {
    let path = device_path(address);
    backend
        .sh(
            "busctl",
            &[
                "--system",
                "call",
                "org.bluez",
                &path,
                "org.bluez.Device1",
                "Disconnect",
            ],
        )
        .await?;
    Ok(())
}

async fn find_adapter(backend: &KdeBackend) -> Option<String> {
    let raw = backend
        .sh(
            "busctl",
            &[
                "--json=short",
                "--system",
                "call",
                "org.bluez",
                "/",
                "org.freedesktop.DBus.ObjectManager",
                "GetManagedObjects",
            ],
        )
        .await
        .ok()?;
    let managed: serde_json::Value = serde_json::from_str(&raw).ok()?;
    find_adapter_path(&managed)
}
