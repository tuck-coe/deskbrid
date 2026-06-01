use super::WayfireBackend;
use crate::protocol::KeyboardLayout;

impl WayfireBackend {
    /// Run setxkbmap and return stdout.
    async fn xkb(&self, args: &[&str]) -> anyhow::Result<String> {
        let mut cmd = tokio::process::Command::new("setxkbmap");
        cmd.args(args)
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped());
        self.apply_env(&mut cmd);
        let out = cmd.output().await?;
        if !out.status.success() {
            anyhow::bail!(
                "setxkbmap failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(String::from_utf8(out.stdout)?.trim().to_string())
    }

    fn parse_query(output: &str) -> Vec<KeyboardLayout> {
        let mut layout = String::new();
        let mut variant = String::new();
        for line in output.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("layout:") {
                layout = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("variant:") {
                variant = val.trim().to_string();
            }
        }
        if layout.is_empty() {
            return vec![];
        }
        vec![KeyboardLayout {
            index: 0,
            name: layout,
            variant: if variant.is_empty() {
                None
            } else {
                Some(variant)
            },
            display_name: None,
        }]
    }

    pub(super) async fn keyboard_layout_list(&self) -> anyhow::Result<Vec<KeyboardLayout>> {
        let out = self.xkb(&["-query"]).await?;
        Ok(Self::parse_query(&out))
    }

    pub(super) async fn keyboard_layout_get(&self) -> anyhow::Result<KeyboardLayout> {
        let layouts = self.keyboard_layout_list().await?;
        layouts
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no keyboard layout found"))
    }

    pub(super) async fn keyboard_layout_set(
        &self,
        _index: Option<u32>,
        name: Option<&str>,
        variant: Option<&str>,
    ) -> anyhow::Result<()> {
        let layout_name = name.unwrap_or("us");
        let mut args: Vec<&str> = vec!["-layout", layout_name];
        if let Some(v) = variant {
            args.push("-variant");
            args.push(v);
        }
        self.xkb(&args).await?;
        Ok(())
    }

    pub(super) async fn keyboard_layout_add(
        &self,
        name: &str,
        variant: Option<&str>,
    ) -> anyhow::Result<()> {
        self.keyboard_layout_set(None, Some(name), variant).await
    }

    pub(super) async fn keyboard_layout_remove(&self, _index: u32) -> anyhow::Result<()> {
        self.xkb(&["-layout", "us"]).await?;
        Ok(())
    }
}
