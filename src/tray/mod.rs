//! System tray icon for Deskbrid — shows update status, controls the daemon,
//! and provides quick actions via StatusNotifierItem (KDE/GNOME/XFCE/etc.).

mod state;
mod ui;

pub use ui::run;
