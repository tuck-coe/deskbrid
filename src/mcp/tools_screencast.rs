// TESTING_NEEDED: This feature requires manual testing on a live desktop environment

#[macro_export]
macro_rules! tools_screencast {
    () => {
    #[tool(
        name = "screencast_start",
        description = "Start recording the desktop to a video file via PipeWire/gst-launch.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    fn screencast_start(
        &self,
        Parameters(ScreencastStartParams { output_path }): Parameters<ScreencastStartParams>,
    ) -> Json<Value> {
        execute(
            self.state.clone(),
            &self.rt,
            "screencast.start",
            json!({ "output_path": output_path }),
        )
    }

    #[tool(
        name = "screencast_stop",
        description = "Stop the running screencast recording.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    fn screencast_stop(&self) -> Json<Value> {
        execute(self.state.clone(), &self.rt, "screencast.stop", json!({}))
    }
    };
}
