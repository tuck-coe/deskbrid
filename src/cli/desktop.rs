use clap::Subcommand;

#[derive(Subcommand)]
pub enum DesktopCmd {
    /// List all available gsettings schemas
    ListSchemas,
    /// Read a desktop setting (e.g. org.gnome.desktop.interface gtk-theme)
    GetSetting {
        /// GSettings schema
        schema: String,
        /// Schema key
        key: String,
    },
    /// Write a desktop setting (e.g. org.gnome.desktop.interface gtk-theme Adwaita)
    SetSetting {
        /// GSettings schema
        schema: String,
        /// Schema key
        key: String,
        /// Value to set
        value: String,
    },
}
