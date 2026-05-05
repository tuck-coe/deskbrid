//! Input injection — keyboard, mouse, and text input via Mutter.RemoteDesktop.
//!
//! Uses the org.gnome.Mutter.RemoteDesktop DBus API to create a session
//! and inject keystrokes/mouse events. This is the Wayland-native equivalent
//! of what xdotool does on X11.

use anyhow::Result;
use serde::Serialize;

/// Manages a Mutter Remote Desktop session for input injection.
#[derive(Clone)]
pub struct InputSession {
    // TODO: Store session path, PipeWire fd, device handles
}

impl InputSession {
    /// Create a new Remote Desktop session.
    pub async fn new() -> Result<Self> {
        // TODO: Call org.gnome.Mutter.RemoteDesktop.CreateSession()
        // TODO: Request keyboard + mouse device types (bitmask 3 or 7)
        // TODO: Store session object path for later use
        Ok(Self)
    }

    /// Type text into the currently focused window.
    pub async fn type_text(&self, _text: &str) -> Result<()> {
        // TODO: Split text into characters, send key press/release via RemoteDesktop
        // TODO: Handle special sequences like \n → Return key
        Ok(())
    }

    /// Send key combinations (e.g., Ctrl+Shift+T).
    pub async fn send_keys(&self, _keys: &[String]) -> Result<()> {
        // TODO: Map key names to evdev keycodes
        // TODO: Press all modifiers, press main key, release in reverse order
        Ok(())
    }

    /// Perform a mouse action (click, move, scroll).
    pub async fn mouse_action(&self, _kind: &str, _x: f64, _y: f64) -> Result<()> {
        // TODO: Call RemoteDesktop.NotifyMouse / NotifyAxis
        Ok(())
    }
}
