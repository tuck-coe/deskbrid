use crate::backend::gnome::GnomeBackend;
use anyhow::Context;

impl GnomeBackend {
    /// Start recording the existing PipeWire ScreenCast stream to an MP4 file.
    /// Spawns a gst-launch-1.0 child process. Only one recording at a time.
    pub(crate) async fn start_screencast(&self, output_path: &str) -> anyhow::Result<()> {
        let mut child_guard = self.sc_child.lock().await;
        if child_guard.is_some() {
            anyhow::bail!("screencast already recording — stop first");
        }
        if self.sc_pw_node == 0 {
            anyhow::bail!("no PipeWire ScreenCast node available");
        }

        let child = tokio::process::Command::new("gst-launch-1.0")
            .args([
                "-q",
                "pipewiresrc",
                &format!("path={}", self.sc_pw_node),
                "!",
                "videoconvert",
                "!",
                "x264enc",
                "!",
                "mp4mux",
                "!",
                "filesink",
                &format!("location={}", output_path),
            ])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .with_context(|| "spawning gst-launch-1.0 for screencast")?;

        *child_guard = Some(child);
        Ok(())
    }

    /// Stop the running screencast. Kills the gst-launch-1.0 child process.
    pub(crate) async fn stop_screencast(&self) -> anyhow::Result<()> {
        let mut child_guard = self.sc_child.lock().await;
        match child_guard.take() {
            Some(mut child) => {
                child.kill().await.context("killing screencast process")?;
                Ok(())
            }
            None => anyhow::bail!("no screencast is running"),
        }
    }
}
