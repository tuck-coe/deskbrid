//! Screen capture — phase 1 screenshot stub using external tools.

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::process::Command;

pub async fn screenshot(_monitor: Option<u32>) -> Result<String> {
    let directory = PathBuf::from("/tmp/deskbrid");
    tokio::fs::create_dir_all(&directory)
        .await
        .context("creating screenshot output dir")?;

    let filename = format!("screenshot_{}.png", unix_ts());
    let path = directory.join(filename);

    let gnome = Command::new("gnome-screenshot")
        .arg("-f")
        .arg(&path)
        .output()
        .await;
    match gnome {
        Ok(output) if output.status.success() => {
            return Ok(path.display().to_string());
        }
        Ok(_) | Err(_) => {}
    }

    let grim_output = Command::new("grim")
        .arg(&path)
        .output()
        .await
        .context("running grim fallback")?;
    if !grim_output.status.success() {
        return Err(anyhow!(
            "grim failed: {}",
            String::from_utf8_lossy(&grim_output.stderr)
        ));
    }

    Ok(path.display().to_string())
}

pub async fn start_screencast(_monitor: u32, _framerate: u32) -> Result<u32> {
    Ok(0)
}

pub async fn stop_screencast(_node_id: u32) -> Result<()> {
    Ok(())
}

fn unix_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
