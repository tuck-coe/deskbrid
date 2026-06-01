use crate::backend::gnome::GnomeBackend;
use anyhow::Context;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

const PORTAL_SCREENSHOT_SCRIPT: &str = include_str!("../../../../scripts/screenshot_portal.py");

/// Run GNOME fallback screenshot methods in priority order.
/// Caller should handle cropping after this returns.
pub(crate) async fn run_fallbacks(backend: &GnomeBackend, path: &str) -> anyhow::Result<()> {
    // Fallback 1: GNOME Shell extension (may hang on GNOME 47+)
    let ext_ok = timeout(
        Duration::from_secs(5),
        screenshot_via_extension(backend, path),
    )
    .await;

    match ext_ok {
        Ok(Ok(())) => Ok(()), // Extension worked
        _ => {
            // Fallback 2: XDG Desktop Portal (ScreenCast)
            screenshot_via_portal(path)
                .await
                .context("all screenshot methods failed (grim, extension, portal)")?;
            Ok(())
        }
    }
}

/// Fallback: screenshot via XDG Desktop Portal (ScreenCast).
/// Uses an external Python script that talks PipeWire.
async fn screenshot_via_portal(output_path: &str) -> anyhow::Result<()> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(PORTAL_SCREENSHOT_SCRIPT)
        .arg(output_path)
        .output()
        .await
        .context("running portal screenshot script")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("portal script failed: {}", stderr);
    }
    Ok(())
}

/// Take a screenshot via the deskbrid GNOME Shell extension.
/// The extension runs inside GNOME Shell and has access to its screenshot API.
/// NOTE: This method may hang on GNOME 47+ — callers should use a timeout.
async fn screenshot_via_extension(backend: &GnomeBackend, output_path: &str) -> anyhow::Result<()> {
    const DBUS_SERVICE: &str = "org.deskbrid.WindowManager";
    const DBUS_PATH: &str = "/org/deskbrid/WindowManager";
    const DBUS_IFACE: &str = "org.deskbrid.WindowManager";

    let reply = backend
        .conn
        .call_method(
            Some(DBUS_SERVICE),
            DBUS_PATH,
            Some(DBUS_IFACE),
            "Screenshot",
            &(output_path,),
        )
        .await
        .map_err(|e| anyhow::anyhow!("extension Screenshot call failed: {e}"))?;

    let success: bool = reply.body().deserialize()?;
    if !success {
        anyhow::bail!("extension screenshot returned false");
    }
    Ok(())
}
