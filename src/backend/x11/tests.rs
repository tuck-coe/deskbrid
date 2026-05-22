use super::helpers::*;

#[test]
fn parses_wmctrl_lgpx_line_with_class_pid_geometry_and_title() {
    let window = parse_wmctrl_line(
        "0x03e00003  2  1234  10  20  1280  720  Navigator.firefox  workstation  Mozilla Firefox",
        Some("65011715"),
    )
    .expect("window");

    assert_eq!(window.id, "0x03e00003");
    assert_eq!(window.workspace_id, 2);
    assert_eq!(window.pid, Some(1234));
    assert_eq!(window.app_id, "firefox");
    assert_eq!(window.title, "Mozilla Firefox");
    assert!(window.is_focused);
    let geometry = window.geometry.expect("geometry");
    assert_eq!(geometry.x, 10);
    assert_eq!(geometry.y, 20);
    assert_eq!(geometry.width, 1280);
    assert_eq!(geometry.height, 720);
}

#[test]
fn parses_wmctrl_sticky_workspace_and_empty_title() {
    let window = parse_wmctrl_line(
        "0x02000001 -1  4321  -5  0  640  480  code.Code  workstation",
        None,
    )
    .expect("window");

    assert_eq!(window.workspace_id, 0);
    assert_eq!(window.app_id, "code");
    assert_eq!(window.title, "");
    assert!(!window.is_focused);
}

#[test]
fn normalizes_decimal_and_hex_window_ids() {
    assert_eq!(normalize_window_id("65011715"), "0x03e00003");
    assert_eq!(normalize_window_id("0X03E00003"), "0x03e00003");
}

#[test]
fn parses_active_xrandr_rotation_before_capability_list() {
    assert_eq!(
        parse_xrandr_rotation(
            "DP-1 connected primary 2560x1440+0+0 right (normal left inverted right x axis y axis)"
        ),
        "right"
    );
    assert_eq!(
        parse_xrandr_rotation(
            "HDMI-1 connected 1080x1920+2560+0 inverted (normal left inverted right x axis y axis)"
        ),
        "inverted"
    );
    assert_eq!(
        parse_xrandr_rotation(
            "eDP-1 connected 1920x1080+0+0 (normal left inverted right x axis y axis)"
        ),
        "normal"
    );
}
