use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct WindowInfo {
    pub window_id: u64,
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub focused: bool,
    pub minimized: bool,
    pub maximized: bool,
    pub fullscreen: bool,
}
