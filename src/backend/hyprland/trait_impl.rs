use super::*;
use crate::backend::DesktopBackend;
use crate::protocol;
use async_trait::async_trait;

// ─── DesktopBackend trait impl ──────────────────────────

#[async_trait]
impl DesktopBackend for HyprBackend {
    async fn windows_list(&self) -> anyhow::Result<Vec<protocol::WindowInfo>> {
        let json = self.hyprctl_json(&["clients"]).await?;
        let arr = json
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("expected JSON array"))?;
        Ok(arr.iter().map(Self::hyprctl_client_to_window).collect())
    }

    async fn window_focus(&self, id: &str) -> anyhow::Result<()> {
        let target = self.resolve_window(id).await?;
        self.hyprctl_dispatch(&format!("focuswindow address:{}", target.id))
            .await
    }

    async fn window_get(&self, id: &str) -> anyhow::Result<protocol::WindowInfo> {
        self.resolve_window(id).await
    }

    async fn window_close(&self, id: &str) -> anyhow::Result<()> {
        let target = self.resolve_window(id).await?;
        self.hyprctl_dispatch(&format!("closewindow address:{}", target.id))
            .await
    }

    async fn window_minimize(&self, _id: &str) -> anyhow::Result<()> {
        anyhow::bail!("Hyprland does not expose a native minimize dispatcher")
    }

    async fn window_maximize(&self, id: &str) -> anyhow::Result<()> {
        let target = self.resolve_window(id).await?;
        self.hyprctl_dispatch(&format!("focuswindow address:{}", target.id))
            .await?;
        if self
            .hyprctl_dispatch("fullscreenstate 1 1 set")
            .await
            .is_ok()
        {
            return Ok(());
        }
        if self.window_is_fullscreen(&target.id).await.unwrap_or(false) {
            return Ok(());
        }
        self.hyprctl_dispatch("fullscreenstate 1 1").await
    }

    async fn window_move_resize(
        &self,
        id: &str,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let target = self.resolve_window(id).await?;
        self.hyprctl_dispatch(&format!(
            "movewindowpixel exact {} {},address:{}",
            x, y, target.id
        ))
        .await?;
        self.hyprctl_dispatch(&format!(
            "resizewindowpixel exact {} {},address:{}",
            width, height, target.id
        ))
        .await
    }

    async fn workspaces_list(&self) -> anyhow::Result<Vec<protocol::WorkspaceInfo>> {
        let json = self.hyprctl_json(&["workspaces"]).await?;
        let arr = json
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("expected array"))?;
        Ok(arr
            .iter()
            .map(|w| protocol::WorkspaceInfo {
                id: w.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as u32,
                name: w
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                is_active: w.get("monitor").and_then(|v| v.as_str()).is_some(),
            })
            .collect())
    }

    async fn workspace_switch(&self, id: u32) -> anyhow::Result<()> {
        self.hyprctl_dispatch(&format!("workspace {}", id)).await
    }

    async fn workspace_move_window(
        &self,
        window_id: &str,
        workspace_id: u32,
        _follow: bool,
    ) -> anyhow::Result<()> {
        let target = self.resolve_window(window_id).await?;
        self.hyprctl_dispatch(&format!(
            "movetoworkspacesilent {},address:{}",
            workspace_id, target.id
        ))
        .await
    }

    async fn keyboard_type(&self, text: &str) -> anyhow::Result<()> {
        self.sh("ydotool", &["type", text]).await?;
        Ok(())
    }
    async fn keyboard_key(&self, key: &str) -> anyhow::Result<()> {
        let k = ydotool_key_name(key);
        self.sh("ydotool", &["key", &k]).await?;
        Ok(())
    }

    async fn keyboard_combo(&self, keys: &[String]) -> anyhow::Result<()> {
        if keys.is_empty() {
            return Ok(());
        }
        let combo: Vec<String> = keys.iter().map(|k| ydotool_key_name(k)).collect();
        for (i, key) in combo.iter().enumerate() {
            if i < combo.len() - 1 {
                self.sh("ydotool", &["key", &format!("{}:1", key)]).await?;
            } else {
                self.sh("ydotool", &["key", key]).await?;
            }
        }
        for key in combo.iter().take(combo.len().saturating_sub(1)) {
            self.sh("ydotool", &["key", &format!("{}:0", key)]).await?;
        }
        Ok(())
    }

    async fn mouse_move(&self, x: f64, y: f64) -> anyhow::Result<()> {
        let (last_x, last_y) = {
            let pos = self.last_mouse.lock().unwrap();
            *pos
        };
        let _dx = x - last_x;
        let _dy = y - last_y;
        {
            let mut pos = self.last_mouse.lock().unwrap();
            *pos = (x, y);
        }
        self.sh(
            "ydotool",
            &[
                "mousemove",
                "--absolute",
                &format!("{}", x as i32),
                &format!("{}", y as i32),
            ],
        )
        .await?;
        Ok(())
    }

    async fn mouse_click(&self, button: &str) -> anyhow::Result<()> {
        let btn_id = match button {
            "left" => "0",
            "middle" => "1",
            "right" => "2",
            _ => anyhow::bail!("unknown button: {}", button),
        };
        self.sh("ydotool", &["click", btn_id]).await?;
        Ok(())
    }

    async fn mouse_scroll(&self, dx: f64, dy: f64) -> anyhow::Result<()> {
        if dx == 0.0 && dy == 0.0 {
            return Ok(());
        }
        self.sh(
            "ydotool",
            &[
                "mousemove",
                "--wheel",
                &format!("{}", dx as i32),
                &format!("{}", dy as i32),
            ],
        )
        .await?;
        Ok(())
    }

    async fn clipboard_read(&self) -> anyhow::Result<String> {
        self.sh("wl-paste", &[]).await
    }

    async fn clipboard_write(&self, text: &str) -> anyhow::Result<()> {
        let mut child = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?;
        use tokio::io::AsyncWriteExt;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).await?;
        }
        let status = child.wait().await?;
        if !status.success() {
            anyhow::bail!("wl-copy failed");
        }
        Ok(())
    }

    async fn screenshot(
        &self,
        monitor: Option<u32>,
        region: Option<protocol::Region>,
        window_id: Option<String>,
    ) -> anyhow::Result<protocol::ScreenshotResult> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let path = format!("/tmp/deskbrid_screenshot_{}.png", ts);
        if let Some(ref wid) = window_id {
            let info = self.window_get(wid).await?;
            if let Some(geo) = info.geometry {
                let region_str = format!("{}x{}+{}+{}", geo.width, geo.height, geo.x, geo.y);
                self.sh("grim", &["-g", &region_str, &path]).await?;
                return Ok(protocol::ScreenshotResult {
                    path: path.clone(),
                    width: geo.width,
                    height: geo.height,
                    format: "png".into(),
                });
            }
        }
        if let Some(ref r) = region {
            let region_str = format!("{}x{}+{}+{}", r.width, r.height, r.x, r.y);
            self.sh("grim", &["-g", &region_str, &path]).await?;
            return Ok(protocol::ScreenshotResult {
                path: path.clone(),
                width: r.width,
                height: r.height,
                format: "png".into(),
            });
        }
        if let Some(idx) = monitor {
            let monitors = {
                let m = self.monitors.lock().unwrap();
                m.clone()
            };
            let name = monitors
                .get(idx as usize)
                .map(|m| m.name.clone())
                .unwrap_or_else(|| idx.to_string());
            self.sh("grim", &["-o", &name, &path]).await?;
        } else {
            self.sh("grim", &[&path]).await?;
        }
        let dims = get_png_dimensions(&path)?;
        Ok(protocol::ScreenshotResult {
            path,
            width: dims.0,
            height: dims.1,
            format: "png".into(),
        })
    }

    async fn notification_send(
        &self,
        app_name: &str,
        title: &str,
        body: &str,
        urgency: &str,
    ) -> anyhow::Result<u32> {
        let urgency_byte = match urgency {
            "low" => "low",
            "normal" => "normal",
            "critical" => "critical",
            _ => "normal",
        };
        let output = self
            .sh(
                "notify-send",
                &[
                    "--app-name",
                    app_name,
                    "--urgency",
                    urgency_byte,
                    "--print-id",
                    title,
                    body,
                ],
            )
            .await?;
        Ok(output.parse().unwrap_or(0))
    }

    async fn notification_close(&self, id: u32) -> anyhow::Result<()> {
        if self.sh_ok("makoctl", &["dismiss", &id.to_string()]).await {
            return Ok(());
        }
        Ok(())
    }

    async fn system_info(&self) -> anyhow::Result<protocol::SystemInfo> {
        let version = self
            .hyprctl_json(&["version"])
            .await
            .map(|v| {
                v.get("version")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown")
                    .to_string()
            })
            .unwrap_or_else(|_| "unknown".into());
        let session_type = if self.wl_socket.is_some() {
            "wayland"
        } else if std::env::var("DISPLAY").is_ok() {
            "x11"
        } else {
            "unknown"
        };
        let monitors = {
            let m = self.monitors.lock().unwrap();
            m.clone()
        };
        let workspaces = self.workspaces_list().await.unwrap_or_default();
        let current_workspace = workspaces
            .iter()
            .find(|w| w.is_active)
            .map(|w| w.id)
            .unwrap_or(1);
        Ok(protocol::SystemInfo {
            desktop: "Hyprland".into(),
            desktop_version: version,
            compositor: "hyprland".into(),
            session_type: session_type.into(),
            monitors,
            workspace_count: workspaces.len() as u32,
            current_workspace,
            idle_seconds: self.idle_seconds_inner().await.unwrap_or(0),
        })
    }

    async fn idle_seconds(&self) -> anyhow::Result<u64> {
        self.idle_seconds_inner().await
    }

    async fn power_action(&self, action: &str) -> anyhow::Result<()> {
        match action {
            "suspend" => {
                self.sh("systemctl", &["suspend"]).await?;
            }
            "hibernate" => {
                self.sh("systemctl", &["hibernate"]).await?;
            }
            "shutdown" | "poweroff" => {
                self.sh("systemctl", &["poweroff"]).await?;
            }
            "reboot" | "restart" => {
                self.sh("systemctl", &["reboot"]).await?;
            }
            "lock" => {
                if !self.sh_ok("loginctl", &["lock-session"]).await {
                    self.sh("hyprctl", &["dispatch", "exec", "loginctl lock-session"])
                        .await?;
                }
            }
            "logout" => {
                self.sh("hyprctl", &["dispatch", "exit"]).await?;
            }
            _ => anyhow::bail!("unsupported power action: {}", action),
        }
        Ok(())
    }

    async fn battery_status(&self) -> anyhow::Result<Vec<protocol::BatteryInfo>> {
        let mut batteries = Vec::new();
        let mut dirs = if let Ok(entries) = tokio::fs::read_dir("/sys/class/power_supply").await {
            entries
        } else {
            return Ok(batteries);
        };
        while let Some(entry) = dirs.next_entry().await? {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with("BAT") {
                continue;
            }
            let capacity = tokio::fs::read_to_string(path.join("capacity"))
                .await
                .ok()
                .and_then(|s| s.trim().parse::<f64>().ok())
                .unwrap_or(0.0);
            let status = tokio::fs::read_to_string(path.join("status"))
                .await
                .ok()
                .map(|s| s.trim().to_lowercase())
                .unwrap_or_else(|| "unknown".into());
            let energy_now = tokio::fs::read_to_string(path.join("energy_now"))
                .await
                .ok()
                .and_then(|s| s.trim().parse::<f64>().ok())
                .unwrap_or(0.0);
            let power_now = tokio::fs::read_to_string(path.join("power_now"))
                .await
                .ok()
                .and_then(|s| s.trim().parse::<f64>().ok())
                .unwrap_or(0.0);
            let time_remaining = if power_now > 0.0 {
                Some(((energy_now / power_now) * 60.0) as u32)
            } else {
                None
            };
            batteries.push(protocol::BatteryInfo {
                source: name.to_string(),
                percentage: capacity,
                state: status,
                time_remaining_minutes: time_remaining,
            });
        }
        Ok(batteries)
    }

    async fn network_status(&self) -> anyhow::Result<protocol::NetworkStatusInfo> {
        let online = if self
            .sh_ok("nmcli", &["networking", "connectivity", "check"])
            .await
        {
            true
        } else {
            self.sh_ok("ping", &["-c", "1", "-W", "2", "8.8.8.8"]).await
        };
        Ok(protocol::NetworkStatusInfo {
            online,
            net_type: if online {
                "wifi_or_ethernet".into()
            } else {
                "offline".into()
            },
        })
    }

    async fn network_interfaces(&self) -> anyhow::Result<Vec<protocol::NetworkInterfaceInfo>> {
        let output = self
            .sh(
                "nmcli",
                &["-t", "-f", "DEVICE,STATE,IP4.ADDRESS", "dev", "status"],
            )
            .await
            .unwrap_or_default();
        let mut ifaces = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() < 2 {
                continue;
            }
            let name = parts[0].to_string();
            if name == "lo" || name.is_empty() {
                continue;
            }
            let state = match *parts.get(1).unwrap_or(&"") {
                "connected" => "connected".to_string(),
                "connecting" => "connecting".to_string(),
                _ => "disconnected".to_string(),
            };
            let ipv4 = parts
                .get(2)
                .filter(|s| !s.is_empty())
                .map(|s| s.split('/').next().unwrap_or(s).to_string());
            ifaces.push(protocol::NetworkInterfaceInfo {
                name,
                state,
                ipv4,
                ipv6: None,
            });
        }
        Ok(ifaces)
    }

    async fn wifi_scan(&self) -> anyhow::Result<Vec<protocol::WifiNetworkInfo>> {
        self.sh("nmcli", &["dev", "wifi", "rescan"]).await.ok();
        let output = self
            .sh(
                "nmcli",
                &["-t", "-f", "SSID,SIGNAL,SECURITY", "dev", "wifi", "list"],
            )
            .await
            .unwrap_or_default();
        let mut networks = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.is_empty() || parts[0].is_empty() {
                continue;
            }
            let ssid = parts[0].to_string();
            let signal: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let security = parts.get(2).unwrap_or(&"").to_string();
            networks.push(protocol::WifiNetworkInfo {
                ssid,
                strength: signal,
                secured: !security.is_empty() && security != "--",
                frequency: None,
            });
        }
        Ok(networks)
    }

    async fn wifi_connect(&self, ssid: &str, password: Option<&str>) -> anyhow::Result<()> {
        match password {
            Some(pw) => {
                self.sh("nmcli", &["dev", "wifi", "connect", ssid, "password", pw])
                    .await?;
            }
            None => {
                self.sh("nmcli", &["dev", "wifi", "connect", ssid]).await?;
            }
        }
        Ok(())
    }

    async fn bluetooth_list(&self) -> anyhow::Result<Vec<protocol::BluetoothDeviceInfo>> {
        let output = self
            .sh("bluetoothctl", &["devices"])
            .await
            .unwrap_or_default();
        let mut devices = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() >= 3 {
                devices.push(protocol::BluetoothDeviceInfo {
                    name: parts[2].to_string(),
                    address: parts[1].to_string(),
                    connected: false,
                    paired: true,
                    rssi: None,
                });
            }
        }
        Ok(devices)
    }

    async fn bluetooth_scan(&self, _duration: Option<u32>) -> anyhow::Result<()> {
        self.sh("bluetoothctl", &["scan", "on"]).await.ok();
        Ok(())
    }
    async fn bluetooth_stop_scan(&self) -> anyhow::Result<()> {
        self.sh("bluetoothctl", &["scan", "off"]).await.ok();
        Ok(())
    }
    async fn bluetooth_connect(&self, address: &str) -> anyhow::Result<()> {
        self.sh("bluetoothctl", &["connect", address]).await?;
        Ok(())
    }
    async fn bluetooth_disconnect(&self, address: &str) -> anyhow::Result<()> {
        self.sh("bluetoothctl", &["disconnect", address]).await?;
        Ok(())
    }

    async fn files_watch(
        &self,
        path: &str,
        recursive: bool,
        _patterns: Option<&[String]>,
    ) -> anyhow::Result<()> {
        use notify::*;
        let event_tx = self.event_tx.clone();
        let watch_path = path.to_string();
        let recursive_mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    let path_str = event.paths.first().map(|p| p.to_string_lossy().to_string());
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if event.kind.is_create() {
                        let _ = event_tx.send(DeskbridEvent::FileCreated {
                            path: path_str.unwrap_or_default(),
                            timestamp: ts,
                        });
                    } else if event.kind.is_modify() {
                        let _ = event_tx.send(DeskbridEvent::FileModified {
                            path: path_str.unwrap_or_default(),
                            timestamp: ts,
                        });
                    } else if event.kind.is_remove() {
                        let _ = event_tx.send(DeskbridEvent::FileDeleted {
                            path: path_str.unwrap_or_default(),
                            timestamp: ts,
                        });
                    }
                }
            })?;
        watcher.watch(std::path::Path::new(&watch_path), recursive_mode)?;
        let mut watchers = self.watchers.lock().unwrap();
        watchers.insert(watch_path, watcher);
        Ok(())
    }

    async fn files_unwatch(&self, path: &str) -> anyhow::Result<()> {
        let mut watchers = self.watchers.lock().unwrap();
        watchers.remove(path);
        Ok(())
    }

    async fn files_search(
        &self,
        pattern: &str,
        root: Option<&str>,
        max_results: u32,
    ) -> anyhow::Result<Vec<String>> {
        let root_path = root.unwrap_or(".");
        let output = self
            .sh(
                "find",
                &[root_path, "-type", "f", "-name", pattern, "-maxdepth", "5"],
            )
            .await
            .unwrap_or_default();
        Ok(output
            .lines()
            .take(max_results as usize)
            .map(|l| l.to_string())
            .collect())
    }

    async fn audio_list_sinks(&self) -> anyhow::Result<Vec<protocol::AudioSinkInfo>> {
        let output = self
            .sh("pactl", &["list", "short", "sinks"])
            .await
            .unwrap_or_default();
        let mut sinks = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                sinks.push(protocol::AudioSinkInfo {
                    id: parts[0].parse().unwrap_or(0),
                    name: parts[1].to_string(),
                    description: String::new(),
                    volume: 1.0,
                    muted: false,
                });
            }
        }
        Ok(sinks)
    }

    async fn audio_set_sink_volume(&self, sink_id: u32, volume: f64) -> anyhow::Result<()> {
        let vol_pct = (volume * 100.0) as u32;
        self.sh(
            "pactl",
            &[
                "set-sink-volume",
                &sink_id.to_string(),
                &format!("{}%", vol_pct),
            ],
        )
        .await?;
        Ok(())
    }

    async fn monitor_set_primary(&self, _output: &str) -> anyhow::Result<()> {
        anyhow::bail!("Hyprland does not expose a primary monitor setting")
    }

    async fn monitor_set_resolution(
        &self,
        output: &str,
        width: u32,
        height: u32,
        refresh_rate: Option<f64>,
    ) -> anyhow::Result<()> {
        let mut config = self.monitor_config(output).await?;
        config.width = width;
        config.height = height;
        if let Some(refresh_rate) = refresh_rate {
            config.refresh_rate = refresh_rate;
        }
        self.apply_monitor_config(&config).await
    }

    async fn monitor_set_scale(&self, output: &str, scale: f64) -> anyhow::Result<()> {
        let mut config = self.monitor_config(output).await?;
        config.scale = scale;
        self.apply_monitor_config(&config).await
    }

    async fn monitor_set_rotation(&self, output: &str, rotation: &str) -> anyhow::Result<()> {
        let mut config = self.monitor_config(output).await?;
        config.transform = rotation_to_hypr_transform(rotation)?;
        self.apply_monitor_config(&config).await
    }

    async fn monitor_set_enabled(&self, output: &str, enabled: bool) -> anyhow::Result<()> {
        let value = if enabled {
            format!("{},preferred,auto,1", output)
        } else {
            format!("{},disable", output)
        };
        self.hyprctl_keyword("monitor", &value).await?;
        self.refresh_monitors_cache().await;
        Ok(())
    }
}
