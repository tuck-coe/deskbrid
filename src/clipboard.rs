//! Clipboard monitoring and access.
//!
//! Uses wlr-data-control protocol via wayland-client for clipboard events,
//! or falls back to wl-clipboard tools.

use anyhow::Result;

/// Clipboard monitor and accessor.
#[derive(Clone)]
pub struct Monitor;

impl Monitor {
    /// Start monitoring clipboard.
    pub fn new() -> Self {
        // TODO: Connect to wlr-data-control-unstable-v1
        // TODO: Subscribe to clipboard change events
        Self
    }

    /// Read current clipboard content.
    pub async fn read(&self) -> Result<String> {
        // TODO: Use wl-clipboard-rs or wl-paste
        Ok(String::new())
    }

    /// Write to clipboard.
    pub async fn write(&self, _text: &str) -> Result<()> {
        // TODO: Use wl-clipboard-rs or wl-copy
        Ok(())
    }
}
