use crate::backend::DesktopBackend;
use crate::protocol::{Geometry, MonitorInfo};

pub async fn tile_window(
    backend: &dyn DesktopBackend,
    window_id: &str,
    preset: &str,
    monitor: Option<u32>,
    padding: u32,
) -> anyhow::Result<serde_json::Value> {
    let info = backend.system_info().await?;
    let monitor_info = select_monitor(&info.monitors, monitor)?;
    let geometry = preset_geometry(&monitor_info, preset, padding)?;
    backend
        .window_move_resize(
            window_id,
            geometry.x,
            geometry.y,
            geometry.width,
            geometry.height,
        )
        .await?;
    Ok(serde_json::json!({
        "window_id": window_id,
        "preset": preset,
        "monitor": monitor_info.id,
        "geometry": geometry
    }))
}

fn select_monitor(monitors: &[MonitorInfo], requested: Option<u32>) -> anyhow::Result<MonitorInfo> {
    if let Some(id) = requested {
        return monitors
            .iter()
            .find(|monitor| monitor.id == id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("monitor not found: {}", id));
    }
    monitors
        .iter()
        .find(|monitor| monitor.primary && monitor.enabled)
        .or_else(|| monitors.iter().find(|monitor| monitor.enabled))
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("no enabled monitors found"))
}

fn preset_geometry(monitor: &MonitorInfo, preset: &str, padding: u32) -> anyhow::Result<Geometry> {
    let pad = padding.min(monitor.width / 3).min(monitor.height / 3);
    let x = monitor.x + pad as i32;
    let y = monitor.y + pad as i32;
    let width = monitor.width.saturating_sub(pad * 2).max(1);
    let height = monitor.height.saturating_sub(pad * 2).max(1);
    let half_w = (width / 2).max(1);
    let half_h = (height / 2).max(1);

    let geometry = match preset {
        "fill" | "maximize" => Geometry {
            x,
            y,
            width,
            height,
        },
        "left" => Geometry {
            x,
            y,
            width: half_w,
            height,
        },
        "right" => Geometry {
            x: x + half_w as i32,
            y,
            width: width - half_w,
            height,
        },
        "top" => Geometry {
            x,
            y,
            width,
            height: half_h,
        },
        "bottom" => Geometry {
            x,
            y: y + half_h as i32,
            width,
            height: height - half_h,
        },
        "top_left" => Geometry {
            x,
            y,
            width: half_w,
            height: half_h,
        },
        "top_right" => Geometry {
            x: x + half_w as i32,
            y,
            width: width - half_w,
            height: half_h,
        },
        "bottom_left" => Geometry {
            x,
            y: y + half_h as i32,
            width: half_w,
            height: height - half_h,
        },
        "bottom_right" => Geometry {
            x: x + half_w as i32,
            y: y + half_h as i32,
            width: width - half_w,
            height: height - half_h,
        },
        "center" => {
            let centered_w = ((width as f64) * 0.8).round() as u32;
            let centered_h = ((height as f64) * 0.8).round() as u32;
            Geometry {
                x: x + ((width - centered_w) / 2) as i32,
                y: y + ((height - centered_h) / 2) as i32,
                width: centered_w.max(1),
                height: centered_h.max(1),
            }
        }
        _ => anyhow::bail!("unsupported tiling preset: {}", preset),
    };
    Ok(geometry)
}

#[cfg(test)]
mod tests {
    use crate::protocol::MonitorInfo;

    #[test]
    fn computes_right_half_geometry() {
        let monitor = MonitorInfo {
            id: 0,
            name: "main".into(),
            width: 1920,
            height: 1080,
            scale: 1.0,
            primary: true,
            enabled: true,
            x: 0,
            y: 0,
            refresh_rate: None,
            rotation: "normal".into(),
        };
        let geometry = super::preset_geometry(&monitor, "right", 10).unwrap();
        assert_eq!(geometry.x, 960);
        assert_eq!(geometry.y, 10);
        assert_eq!(geometry.width, 950);
        assert_eq!(geometry.height, 1060);
    }
}
