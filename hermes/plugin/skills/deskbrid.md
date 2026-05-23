---
name: deskbrid
description: >
  Full Linux desktop control for Hermes agents — 85 tools across windows,
  keyboard/mouse input, AT-SPI UI inspection, clipboard, screenshots,
  audio, system, files, terminals, browser automation, process management,
  monitor configuration, media playback, notifications, hotkeys, and more.
  Use these tools to automate desktop tasks, interact with GUI apps,
  inspect UI state, and manage the machine.
tools: [list_windows, focused_window, list_workspaces, list_apps,
  get_accessibility_tree, screenshot, screenshot_region, screenshot_diff,
  focus_window, close_window, minimize_window, maximize_window,
  move_resize_window, tile_window, activate_or_launch,
  switch_workspace, move_window_to_workspace,
  type_text, press_key, press_keys, mouse_move, mouse_click, mouse_scroll,
  click_coordinate, drag,
  clipboard_read, clipboard_write,
  perform_action, set_element_value, get_element_text, click_element,
  doctor, setup_accessibility, capabilities,
  system_info, battery_status, idle_seconds, network_status,
  bluetooth_list, bluetooth_scan,
  service_status, service_start, service_stop, journal_query,
  list_audio_sinks, set_volume,
  file_list, file_read, file_write, file_search, file_copy, file_watch,
  terminal_create, terminal_write, terminal_read, terminal_resize,
  layout_list, layout_save, layout_restore, layout_delete,
  list_monitors, set_primary_monitor, set_monitor_resolution,
  set_monitor_scale, set_monitor_rotation, enable_monitor, disable_monitor,
  list_browser_tabs, browser_navigate, browser_evaluate,
  browser_screenshot, browser_click,
  list_media_players, media_player_info, media_player_control,
  list_processes, start_process, stop_process, signal_process,
  process_exists, wait_for_process,
  send_notification, close_notification,
  register_hotkey, unregister_hotkey]
---

# Deskbrid — Linux Desktop Control

You have full control over the Linux desktop through 85 Deskbrid tools.

## Core Workflow

1. **See**: Use `screenshot` to capture the screen
2. **Find**: `list_windows` for window IDs, AT-SPI tools for UI elements
3. **Act**: Keyboard/mouse tools to interact
4. **Verify**: Another screenshot to confirm

## Windows

- `list_windows` — IDs, titles, classes, geometry for all windows
- `focused_window` — Which window is currently active
- `focus_window` — Bring window to foreground by ID
- `close_window`, `minimize_window`, `maximize_window` — Standard ops
- `move_resize_window` — Position (x, y) and size (width, height)
- `tile_window` — Snap to left|right|maximize|fullscreen
- `activate_or_launch` — Focus running app or launch if not running

## Workspaces

- `list_workspaces` — All virtual desktops with current state
- `switch_workspace` — Jump to workspace by index (0-based)
- `move_window_to_workspace` — Send window to workspace, optionally follow

## Keyboard

- `type_text` — Type at human speed (preferred for text)
- `press_key` — Single key: Return, Escape, Tab, F5, Super_L, etc.
- `press_keys` — Combos: `["Control_L", "c"]` for copy

Always click into a text field before typing.

## Mouse

- `mouse_move` — Absolute coordinates (x, y)
- `mouse_click` — left|middle|right at current position
- `mouse_scroll` — Wheel (dx for horizontal, dy for vertical; negative dy = down)
- `click_coordinate` — Move to (x, y) + click
- `drag` — Click-and-drag from (from_x, from_y) to (to_x, to_y)

## Clipboard

- `clipboard_read` — Get clipboard text
- `clipboard_write` — Put text on clipboard (then paste with Control_L+v)

## Screenshots

- `screenshot` — Full screen as base64 PNG
- `screenshot_region` — Capture region (region_x/y/w/h), specific monitor, or window
- `screenshot_diff` — Pixel diff between two screenshots (detect UI changes)

Always screenshot after actions to verify. Always screenshot before to understand state.

## AT-SPI (Accessibility Tree)

- `list_apps` — Running apps with accessibility support
- `get_accessibility_tree` — Full UI tree: bounds, roles, states, actions, text
- `perform_action` — Act on element (click, activate, toggle) by object_ref
- `set_element_value` — Type into an editable AT-SPI element
- `get_element_text` — Read text from an AT-SPI element
- `click_element` — Click element by object_ref (coordinate fallback)

Use `get_accessibility_tree` when you need precise UI structure. Element object_refs
from the tree can be used directly with `click_element` and `set_element_value`.

## Audio

- `list_audio_sinks` — Output devices with volume/mute
- `set_volume` — Set sink volume 0.0–1.0

## System

- `system_info` — Hostname, OS, kernel, uptime, memory, CPU
- `battery_status` — Battery percentage + charging state
- `idle_seconds` — Time since last user input
- `network_status` — Interfaces, IPs, connectivity
- `bluetooth_list` / `bluetooth_scan` — Paired devices / scan nearby
- `service_status` / `service_start` / `service_stop` — systemd unit management
- `journal_query` — Query systemd journal (since, until, unit, priority, tail)

## Diagnostics

- `doctor` — Check AT-SPI readiness + dependency status
- `setup_accessibility` — Enable AT-SPI via gsettings
- `capabilities` — All available Deskbrid action types

## Files

- `file_list` — Directory listing (path)
- `file_read` — Read file contents (path, offset, limit)
- `file_write` — Create/overwrite/append (path, content, append)
- `file_search` — Glob/regex search (pattern, root, max_results)
- `file_copy` — Copy file or directory (source, destination)
- `file_watch` — Watch path for changes (path, recursive, patterns)

File operations respect the daemon's working directory. Use absolute paths when unsure.

## Terminal

- `terminal_create` — Spawn PTY (shell, cwd, rows, cols). Returns terminal_id.
- `terminal_write` — Send input to terminal (supports ANSI escapes)
- `terminal_read` — Read output (max_bytes, flush)
- `terminal_resize` — Change rows/cols

Terminal lifecycle: create → write → read → (write/read loop) → process will continue
running until killed. Terminals are stateful — keep the terminal_id for subsequent calls.

## Layout Profiles

- `layout_list` — List saved window layouts
- `layout_save` — Save current arrangement as named profile (overwrite optional)
- `layout_restore` — Restore windows to saved positions
- `layout_delete` — Remove a saved profile

## Monitor

- `list_monitors` — All connected displays with resolution, position, scale, refresh
- `set_primary_monitor` — Set primary display (output name e.g. DP-1)
- `set_monitor_resolution` — Change resolution + optional refresh rate
- `set_monitor_scale` — Display scaling factor (1.0, 1.5, 2.0)
- `set_monitor_rotation` — normal|left|right|inverted
- `enable_monitor` / `disable_monitor` — Turn outputs on/off

## Browser (Chrome DevTools Protocol)

Requires Chrome/Chromium with `--remote-debugging-port=9222`.

- `list_browser_tabs` — Open tabs
- `browser_navigate` — Go to URL (tab_index, url)
- `browser_evaluate` — Run JavaScript, return result (expression, await_promise)
- `browser_screenshot` — Capture tab as image
- `browser_click` — Click element by CSS selector

## Media (MPRIS)

- `list_media_players` — All MPRIS players on D-Bus
- `media_player_info` — Track, artist, album, position, playback status
- `media_player_control` — play|pause|play_pause|next|previous|stop

## Process

- `list_processes` — Running processes with PID, name, CPU, memory
- `start_process` — Launch background process (command, workdir). Returns PID.
- `stop_process` / `signal_process` — Kill/signal by PID
- `process_exists` — Check if PID is alive
- `wait_for_process` — Block until process exits (optional timeout)

## Notifications

- `send_notification` — Desktop notification (app_name, title, body, urgency)
- `close_notification` — Dismiss by notification_id

Urgency: low, normal, critical.

## Hotkeys

- `register_hotkey` — Bind global key combo to an ID
- `unregister_hotkey` — Remove binding

## Patterns

### Open an App
1. `press_keys(["Super_L"])`
2. `type_text("firefox")`
3. `press_key("Return")`
4. Wait 2 seconds
5. `screenshot()`

### Click by Position
1. `screenshot()`
2. `click_coordinate(x=450, y=320)`
3. `screenshot()`

### Fill a Form
1. `focus_window("firefox-123")`
2. `click_coordinate(x=200, y=150)`
3. `type_text("hello@example.com")`
4. `press_key("Tab")`
5. `type_text("password")`
6. `press_key("Return")`

### Read Clipboard
1. `press_keys(["Control_L", "a"])` — Select all
2. `press_keys(["Control_L", "c"])` — Copy
3. `clipboard_read()` — Read

### Inspect UI (AT-SPI)
1. `list_apps()` — Find the app
2. `get_accessibility_tree(app_name="firefox")` — Get element tree
3. Find the element you need by name/role
4. `click_element(object_ref="...")` or `set_element_value(object_ref="...", value="text")`

### Run a Command
1. `terminal_create(shell="/bin/bash")` → terminal_id: "abc123"
2. `terminal_write(terminal_id="abc123", input="ls -la\n")`
3. `terminal_read(terminal_id="abc123", flush=true)` → output

### Check Service Health
1. `service_status(name="nginx.service")` → active/inactive
2. If down: `service_start(name="nginx.service")`
3. `journal_query(unit="nginx.service", tail=20)` → recent logs

### Diff Screenshots for Changes
1. `screenshot()` → save the output path as "before"
2. Perform the action
3. `screenshot_diff(before_path="/tmp/before.png", tolerance=10)`

## Limitations

- Cannot read text from screen directly (use clipboard patterns or AT-SPI)
- AT-SPI needs app accessibility support (most GTK/Qt apps have it)
- Wayland has stricter input security — some mouse/keyboard tools may need uinput
- Browser tools require Chrome/Chromium with remote debugging enabled
- File operations respect daemon permissions
- Terminal processes persist until killed explicitly
