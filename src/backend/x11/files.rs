use super::*;

pub(super) async fn files_watch(
    _backend: &X11Backend,
    _path: &str,
    _recursive: bool,
    _patterns: Option<&[String]>,
) -> anyhow::Result<()> {
    anyhow::bail!("not implemented on x11 backend")
}

pub(super) async fn files_unwatch(_backend: &X11Backend, _path: &str) -> anyhow::Result<()> {
    anyhow::bail!("not implemented on x11 backend")
}

pub(super) async fn files_search(
    _backend: &X11Backend,
    _pattern: &str,
    _root: Option<&str>,
    _max_results: u32,
) -> anyhow::Result<Vec<String>> {
    Ok(Vec::new())
}
