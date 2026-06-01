#[macro_export]
macro_rules! tools_desktop {
    () => {
    #[tool(
        name = "list_schemas",
        description = "List all available gsettings schemas on the system.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    fn list_schemas(&self) -> Json<Value> {
        block(&self.rt, do_execute(&self.state, "desktop.list_schemas", json!({})))
    }

    #[tool(
        name = "get_setting",
        description = "Read a desktop setting value by schema and key (e.g. 'org.gnome.desktop.interface', 'gtk-theme').",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    fn get_setting(
        &self,
        Parameters(DesktopSettingKey { schema, key }): Parameters<DesktopSettingKey>,
    ) -> Json<Value> {
        execute(
            self.state.clone(),
            &self.rt,
            "desktop.get_setting",
            json!({"schema": schema, "key": key}),
        )
    }

    #[tool(
        name = "set_setting",
        description = "Write a desktop setting value by schema, key, and value.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    fn set_setting(
        &self,
        Parameters(DesktopSettingValue { schema, key, value }): Parameters<DesktopSettingValue>,
    ) -> Json<Value> {
        execute(
            self.state.clone(),
            &self.rt,
            "desktop.set_setting",
            json!({"schema": schema, "key": key, "value": value}),
        )
    }
    };
}
