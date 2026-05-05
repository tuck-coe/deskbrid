//! Screen capture — screenshots and screencasts via Mutter.ScreenCast + PipeWire.
//!
//! Uses the org.gnome.Mutter.ScreenCast portal for full screen/window/area capture,
//! streamed via PipeWire DMA-BUF buffers.

use anyhow::Result;

/// Take a screenshot.
pub async fn screenshot(_monitor: Option<u32>) -> Result<String> {
    // TODO: Create ScreenCast session via DBus
    // TODO: Request a single-frame screencast
    // TODO: Save PipeWire frame to PNG via DMA-BUF → shm copy
    // TODO: Return file path
    Ok("/tmp/deskbrid/screenshot.png".to_string())
}

/// Start a screencast stream.
pub async fn start_screencast(_monitor: u32, _framerate: u32) -> Result<u32> {
    // TODO: Create persistent ScreenCast session
    // TODO: Negotiate PipeWire stream parameters
    // TODO: Return node ID for the stream
    Ok(0)
}

/// Stop a screencast stream.
pub async fn stop_screencast(_node_id: u32) -> Result<()> {
    // TODO: Close PipeWire stream and ScreenCast session
    Ok(())
}
