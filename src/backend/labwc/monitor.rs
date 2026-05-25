use super::*;

pub(super) async fn monitor_set_primary(
    _backend: &LabwcBackend,
    _output: &str,
) -> anyhow::Result<()> {
    anyhow::bail!("Labwc does not expose a primary monitor setting")
}

pub(super) async fn monitor_set_resolution(
    backend: &LabwcBackend,
    output: &str,
    width: u32,
    height: u32,
    refresh_rate: Option<f64>,
) -> anyhow::Result<()> {
    // Try with exact mode string first, fall back to width×height only.
    // wlr-randr requires exact mode matching (e.g. 1366x768@60Hz fails
    // when the display reports 1366x768@60.026Hz).
    let mode = crate::backend::wlr_randr::mode_arg(width, height, refresh_rate);
    let args: Vec<String> = vec!["--output".into(), output.into(), "--mode".into(), mode];
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = backend.sh("wlr-randr", &refs).await;
    if result.is_ok() {
        return Ok(());
    }
    // Fallback: try without refresh rate
    if refresh_rate.is_some() {
        let fallback_mode = crate::backend::wlr_randr::mode_arg(width, height, None);
        let fallback_args: Vec<String> = vec![
            "--output".into(),
            output.into(),
            "--mode".into(),
            fallback_mode,
        ];
        let fallback_refs: Vec<&str> = fallback_args.iter().map(String::as_str).collect();
        backend.sh("wlr-randr", &fallback_refs).await.map(|_| ())
    } else {
        result.map(|_| ())
    }
}

pub(super) async fn monitor_set_scale(
    backend: &LabwcBackend,
    output: &str,
    scale: f64,
) -> anyhow::Result<()> {
    run_wlr_randr(
        backend,
        crate::backend::wlr_randr::set_scale_args(output, scale),
    )
    .await
}

pub(super) async fn monitor_set_rotation(
    backend: &LabwcBackend,
    output: &str,
    rotation: &str,
) -> anyhow::Result<()> {
    run_wlr_randr(
        backend,
        crate::backend::wlr_randr::set_rotation_args(output, rotation)?,
    )
    .await
}

pub(super) async fn monitor_set_enabled(
    backend: &LabwcBackend,
    output: &str,
    enabled: bool,
) -> anyhow::Result<()> {
    run_wlr_randr(
        backend,
        crate::backend::wlr_randr::set_enabled_args(output, enabled),
    )
    .await
}

async fn run_wlr_randr(backend: &LabwcBackend, args: Vec<String>) -> anyhow::Result<()> {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    backend.sh("wlr-randr", &refs).await.map(|_| ())
}
