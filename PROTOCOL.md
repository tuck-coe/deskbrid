# Deskbrid Protocol v0.1

> The agent-native Linux desktop protocol.

Deskbrid is a JSON-over-Unix-socket protocol for desktop event subscription and action execution. Any agent platform (Hermes, Praxis, Claude Code, OpenAI Operator, etc.) can connect and get full desktop control.

## Transport

- **Default socket**: `$XDG_RUNTIME_DIR/deskbrid/socket` (typically `/run/user/1000/deskbrid/socket`)
- **Protocol**: JSON lines (`\n`-delimited), bidirectional
- **Encoding**: UTF-8
- **Max message size**: 1 MiB

### Connection lifecycle

1. Agent opens a Unix stream socket connection to the deskbrid daemon
2. Daemon sends initial `server:hello` with version info
3. Agent sends subscribe/action messages
4. Daemon pushes event messages as they occur
5. Either side may close the connection

```
→ (connect)
← {"type":"hello","version":"0.1","pid":1234}
→ {"type":"subscribe","events":["window:focus","clipboard","notifications"]}
← {"type":"event","event":"window:focus","data":{...}}
← {"type":"event","event":"notifications","data":{...}}
→ {"type":"action","action":"inject:type","params":{"text":"hello"}}
← {"type":"result","ok":true,"id":"...","data":{}}
```

## Message Types

### Server → Client

#### `hello`
Sent on connection.
```json
{"type": "hello", "version": "0.1", "pid": 1234}
```

#### `event`
Asynchronous event pushed to subscribers.
```json
{"type": "event", "event": "<event_type>", "data": {...}}
```

#### `result`
Response to a client action request.
```json
{"type": "result", "id": "uuid", "ok": true, "data": {...}}
{"type": "result", "id": "uuid", "ok": false, "error": "reason"}
```

### Client → Server

#### `subscribe`
Register interest in event types.
```json
{"type": "subscribe", "events": ["window:focus", "window:open", "window:close", "clipboard", "notifications", "idle", "audio"]}
```

Response: `{"type":"result","id":"...","ok":true}`

#### `unsubscribe`
Stop receiving an event type.
```json
{"type": "unsubscribe", "events": ["clipboard"]}
```

#### `action`
Perform a desktop action.
```json
{"type": "action", "id": "unique-id", "action": "<action_name>", "params": {...}}
```

## Events

### `window:focus`
Fired when the focused window changes.
```json
{
  "event": "window:focus",
  "data": {
    "title": "Terminal",
    "app_id": "org.gnome.Terminal",
    "pid": 12345,
    "workspace": 1,
    "geometry": [0, 0, 1920, 1080],
    "wm_class": "Gnome-terminal"
  }
}
```

### `window:open`
Fired when a new window appears.
```json
{
  "event": "window:open",
  "data": {
    "title": "Firefox",
    "app_id": "firefox",
    "pid": 54321,
    "workspace": 1,
    "geometry": [100, 100, 1280, 720]
  }
}
```

### `window:close`
Fired when a window is closed.
```json
{
  "event": "window:close",
  "data": {
    "app_id": "firefox",
    "pid": 54321
  }
}
```

### `clipboard`
Fired when clipboard content changes — includes both text and metadata about content type.
```json
{
  "event": "clipboard",
  "data": {
    "text": "git push origin main",
    "mime_types": ["text/plain"],
    "timestamp": 1714892345
  }
}
```

### `notifications`
Fired when a desktop notification appears.
```json
{
  "event": "notifications",
  "data": {
    "app": "Telegram",
    "app_icon": "telegram",
    "summary": "Jeremy Coe",
    "body": "Push the commits",
    "urgency": "normal",
    "id": 42
  }
}
```

### `idle`
Fired when user idle state changes.
```json
{
  "event": "idle",
  "data": {
    "idle": true,
    "idle_since": 1714892345,
    "idle_seconds": 300
  }
}
```

### `audio:node`
Fired when audio nodes change (app started/stopped playing).
```json
{
  "event": "audio:node",
  "data": {
    "id": 67,
    "name": "Firefox",
    "state": "running",
    "volume": 0.75,
    "muted": false
  }
}
```

## Actions

### `window:list`
List all open windows.
```json
{"action": "window:list"}
```
```json
{"ok": true, "data": {
  "windows": [
    {"title": "Terminal", "app_id": "org.gnome.Terminal", "pid": 12345, "workspace": 1, "focused": true}
  ]
}}
```

### `window:focus`
Focus a window by app_id or title (fuzzy match).
```json
{"action": "window:focus", "params": {"app_id": "firefox"}}
{"action": "window:focus", "params": {"title": "Terminal", "exact": false}}
```

### `inject:type`
Type text into the currently focused window.
```json
{"action": "inject:type", "params": {"text": "git push origin main\n"}}
```
Special characters: `\n` = Enter, `\t` = Tab.

### `inject:key`
Send key combinations.
```json
{"action": "inject:key", "params": {"keys": ["ctrl", "shift", "t"]}}
{"action": "inject:key", "params": {"keys": ["alt", "f4"]}}
{"action": "inject:key", "params": {"keys": ["super", "d"]}}
```

### `inject:mouse`
Mouse actions.
```json
{"action": "inject:mouse", "params": {"type": "click", "button": "left", "x": 100, "y": 200}}
{"action": "inject:mouse", "params": {"type": "move", "x": 500, "y": 300}}
{"action": "inject:mouse", "params": {"type": "scroll", "dx": 0, "dy": -3}}
```

### `clipboard:read`
Read current clipboard content.
```json
{"action": "clipboard:read"}
```
```json
{"ok": true, "data": {"text": "content", "mime_types": ["text/plain"]}}
```

### `clipboard:write`
Write to clipboard.
```json
{"action": "clipboard:write", "params": {"text": "content to copy"}}
```

### `screenshot`
Capture the screen (or a specific monitor).
```json
{"action": "screenshot", "params": {"monitor": 0}}
```
```json
{"ok": true, "data": {"path": "/tmp/deskbrid/screenshot_1714892345.png", "width": 1920, "height": 1080}}
```

### `screencast:start` / `screencast:stop`
Start/stop streaming screen content via PipeWire.
```json
{"action": "screencast:start", "params": {"monitor": 0, "framerate": 15}}
```
```json
{"ok": true, "data": {"pipewire_fd": 3, "node_id": 42, "width": 1920, "height": 1080}}
```

### `notification:send`
Send a desktop notification.
```json
{"action": "notification:send", "params": {"summary": "Build Done", "body": "All tests pass", "urgency": "low"}}
```

### `display:list`
List monitors and their configurations.
```json
{"action": "display:list"}
```
```json
{"ok": true, "data": {"monitors": [{"id": 0, "width": 1920, "height": 1080, "scale": 1.0, "refresh": 60}]}}
```

### `info`
Get daemon and desktop capabilities.
```json
{"action": "info"}
```
```json
{"ok": true, "data": {
  "deskbrid_version": "0.1.0",
  "desktop": "GNOME",
  "desktop_version": "42.9",
  "session_type": "wayland",
  "capabilities": ["window", "inject", "clipboard", "screenshot", "screencast", "notifications", "display", "idle", "audio"]
}}
```

## Error Handling

Errors are always returned with `"ok": false`:

```json
{"type": "result", "id": "uuid", "ok": false, "error": "not_subscribed", "message": "Must subscribe to clipboard events first"}
```

Standard error codes:

| Code | Meaning |
|---|---|
| `not_subscribed` | Action requires a subscription first |
| `not_supported` | Desktop doesn't support this capability |
| `permission_denied` | Portal permission not granted |
| `invalid_params` | Missing or malformed parameters |
| `session_error` | RemoteDesktop/ScreenCast session error |
| `internal_error` | Daemon-side failure |

## Versioning

The protocol uses **semantic versioning** for the socket contract. Breaking changes (field removals, required fields) increment the major version. Additive changes (new events, new action types) increment the minor version. The daemon's `version` field in `hello` conveys both.

---

*This is a living document. The protocol evolves with the desktop.*
