pub mod detect;
pub mod gnome;
pub mod kde;
pub mod types;

use crate::events::EventBus;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
pub use types::{MonitorInfo, WindowInfo};

use self::detect::{detect_desktop, DesktopType};
use self::gnome::GnomeBackend;

#[async_trait]
pub trait DesktopBackend: Send + Sync {
    async fn list_windows(&self) -> Result<Vec<WindowInfo>>;
    async fn focus_window(&self, app_id: Option<&str>, title: Option<&str>, exact: bool) -> Result<()>;
    async fn focused_window(&self) -> Result<Option<WindowInfo>>;
    async fn list_displays(&self) -> Result<Vec<MonitorInfo>>;
    async fn create_input_session(&self) -> Result<Box<dyn InputBackend>>;
    async fn send_notification(&self, summary: &str, body: &str, urgency: &str) -> Result<u32>;
    fn desktop_name(&self) -> &'static str;
}

#[async_trait]
pub trait InputBackend: Send + Sync {
    async fn type_text(&self, text: &str) -> Result<()>;
    async fn send_keys(&self, keys: &[String]) -> Result<()>;
    async fn mouse_action(&self, params: &Value) -> Result<()>;
}

pub async fn create_backend(event_bus: EventBus) -> Result<Box<dyn DesktopBackend>> {
    match detect_desktop() {
        DesktopType::Gnome => Ok(Box::new(GnomeBackend::new(event_bus).await?)),
        DesktopType::Kde => Err(anyhow!("KDE backend not yet implemented")),
        DesktopType::Other => Err(anyhow!("unsupported desktop environment")),
    }
}
