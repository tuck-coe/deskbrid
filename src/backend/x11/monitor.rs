use super::*;

pub(super) async fn monitor_set_primary(backend: &X11Backend, output: &str) -> anyhow::Result<()> {
    backend
        .sh("xrandr", &["--output", output, "--primary"])
        .await
        .map(|_| ())
}

pub(super) async fn monitor_set_resolution(
    backend: &X11Backend,
    output: &str,
    width: u32,
    height: u32,
    refresh_rate: Option<f64>,
) -> anyhow::Result<()> {
    let mut args = vec![
        "--output".to_string(),
        output.to_string(),
        "--mode".into(),
        format!("{}x{}", width, height),
    ];
    if let Some(refresh) = refresh_rate {
        args.push("--rate".into());
        args.push(format_monitor_float(refresh));
    }
    backend.sh_owned("xrandr", args).await.map(|_| ())
}

pub(super) async fn monitor_set_scale(
    backend: &X11Backend,
    output: &str,
    scale: f64,
) -> anyhow::Result<()> {
    let scale_arg = format!("{0}x{0}", format_monitor_float(scale));
    backend
        .sh_owned(
            "xrandr",
            vec![
                "--output".into(),
                output.into(),
                "--scale".into(),
                scale_arg,
            ],
        )
        .await
        .map(|_| ())
}

pub(super) async fn monitor_set_rotation(
    backend: &X11Backend,
    output: &str,
    rotation: &str,
) -> anyhow::Result<()> {
    backend
        .sh(
            "xrandr",
            &["--output", output, "--rotate", xrandr_rotation(rotation)?],
        )
        .await
        .map(|_| ())
}

pub(super) async fn monitor_set_enabled(
    backend: &X11Backend,
    output: &str,
    enabled: bool,
) -> anyhow::Result<()> {
    backend
        .sh(
            "xrandr",
            &["--output", output, if enabled { "--auto" } else { "--off" }],
        )
        .await
        .map(|_| ())
}
