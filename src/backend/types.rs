use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct WindowInfo {
    pub title: String,
    pub app_id: String,
    pub pid: i64,
    #[serde(default)]
    pub workspace: i64,
    #[serde(default)]
    pub focused: bool,
    #[serde(default)]
    pub geometry: [i64; 4],
    #[serde(default)]
    pub wm_class: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonitorInfo {
    pub id: u32,
    pub width: i32,
    pub height: i32,
    pub scale: f64,
    pub refresh: u32,
}
