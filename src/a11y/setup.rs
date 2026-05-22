//! Accessibility setup and diagnostics (doctor check).

use serde_json::json;

/// Check if AT-SPI accessibility is enabled via gsettings.
pub async fn check_accessibility_enabled() -> bool {
    let output = tokio::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "accessibility-enable"])
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_lowercase());
    matches!(output, Ok(ref s) if s == "true")
}

/// Enable AT-SPI accessibility via gsettings.
pub async fn enable_accessibility() -> anyhow::Result<bool> {
    let status = tokio::process::Command::new("gsettings")
        .args([
            "set",
            "org.gnome.desktop.interface",
            "accessibility-enable",
            "true",
        ])
        .status()
        .await?;
    Ok(status.success())
}

/// Run a health check and return diagnostic report.
pub async fn doctor_report() -> serde_json::Value {
    let a11y_enabled = check_accessibility_enabled().await;
    let bus_addr = std::env::var("AT_SPI_BUS_ADDRESS").ok();

    let gsettings_ok = a11y_enabled;
    let bus_ok = bus_addr.is_some();

    json!({
        "accessibility_enabled": a11y_enabled,
        "at_spi_bus_address_set": bus_ok,
        "gsettings_ok": gsettings_ok,
        "bus_ok": bus_ok,
        "ready": gsettings_ok || bus_ok,
        "remediation": if !gsettings_ok && !bus_ok {
            Some("Run 'deskbrid setup' to enable accessibility")
        } else {
            None
        }
    })
}
