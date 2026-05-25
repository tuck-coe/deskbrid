/// Sample a pixel from an image file at relative coordinates (x, y).
pub async fn pick_color_from_image(
    path: &str,
    x: u32,
    y: u32,
) -> anyhow::Result<serde_json::Value> {
    let pixel = tokio::task::spawn_blocking({
        let path = path.to_string();
        move || sample_pixel(&path, x, y)
    })
    .await??;

    Ok(serde_json::json!({
        "x": x,
        "y": y,
        "source_path": path,
        "red": pixel[0],
        "green": pixel[1],
        "blue": pixel[2],
        "alpha": pixel[3],
        "hex": rgba_to_hex(pixel)
    }))
}

pub(crate) fn sample_pixel(path: &str, x: u32, y: u32) -> anyhow::Result<[u8; 4]> {
    let image = image::open(path)?.to_rgba8();
    if x >= image.width() || y >= image.height() {
        anyhow::bail!(
            "sample coordinate {},{} outside image bounds {}x{}",
            x,
            y,
            image.width(),
            image.height()
        );
    }
    Ok(image.get_pixel(x, y).0)
}

pub(crate) fn rgba_to_hex(pixel: [u8; 4]) -> String {
    if pixel[3] == 255 {
        format!("#{:02x}{:02x}{:02x}", pixel[0], pixel[1], pixel[2])
    } else {
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            pixel[0], pixel[1], pixel[2], pixel[3]
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn formats_rgb_and_rgba_hex() {
        assert_eq!(super::rgba_to_hex([255, 128, 0, 255]), "#ff8000");
        assert_eq!(super::rgba_to_hex([255, 128, 0, 127]), "#ff80007f");
    }
}
