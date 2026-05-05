//! DBus hub — watches GNOME Shell, Notifications, IdleMonitor, and more.
//!
//! Uses zbus to connect to the session bus and subscribe to signals
//! from various org.gnome.* and org.freedesktop.* services.

use crate::events::EventBus;
use anyhow::Result;

/// Hub manages DBus connections to all desktop services.
#[derive(Clone)]
pub struct Hub {
    // connection held for lifetime
    _conn: zbus::Connection,
}

impl Hub {
    /// Connect to the session bus.
    pub async fn new(_event_bus: EventBus) -> Self {
        let conn = zbus::Connection::session()
            .await
            .expect("failed to connect to session bus");
        Self { _conn: conn }
    }

    /// Watch for window focus/open/close events via GNOME Shell.
    pub async fn watch_windows(self, _event_bus: EventBus) -> Result<()> {
        // TODO: Subscribe to Shell.Introspect.WindowsChanged signal
        // TODO: Poll GetWindows() periodically or use Eval() for focus tracking
        // Current blocker: GetWindows() returns AccessDenied from outside
        // Solution: Use Shell.Eval() with JS: global.display.focus_window.get_title()
        Ok(())
    }

    /// Watch for desktop notifications.
    pub async fn watch_notifications(self, _event_bus: EventBus) -> Result<()> {
        // TODO: Subscribe to org.freedesktop.Notifications.Notify signal
        Ok(())
    }

    /// Watch for user idle state changes.
    pub async fn watch_idle(self, _event_bus: EventBus) -> Result<()> {
        // TODO: Subscribe to org.gnome.Mutter.IdleMonitor signals
        Ok(())
    }
}
