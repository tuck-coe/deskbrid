# System Information

Query system status, power information, and control system functions.

## System Info

```bash
deskbrid system info
```

Response:
```json
{
  "type": "response",
  "status": "ok",
  "data": {
    "desktop": "gnome",
    "desktop_version": "45.0",
    "compositor": "gnome-shell",
    "session_type": "wayland",
    "monitors": [
      {"id": 0, "name": "DP-1", "width": 1920, "height": 1080, "scale": 1.0, "primary": true}
    ],
    "workspace_count": 4,
    "current_workspace": 0
  }
}
```

Protocol:
```json
{"type": "system.info"}
```

## System Capabilities

```bash
deskbrid system capabilities
```

Response:
```json
{
  "type": "response",
  "status": "ok",
  "data": {
    "screenshot": true,
    "ocr": true,
    "clipboard": true,
    "input": true,
    "notifications": true,
    "mpris": true
  }
}
```

Protocol:
```json
{"type": "system.capabilities"}
```

## Power Management

### Power Actions

```bash
deskbrid system power suspend
deskbrid system power reboot
deskbrid system power shutdown
```

Protocol:
```json
{"type": "system.power", "action": "suspend"}
```

Actions:
- `suspend` - Suspend to RAM
- `hibernate` - Suspend to disk
- `reboot` - Reboot system
- `shutdown` - Power off
- `lock` - Lock screen

## Battery Status

```bash
deskbrid system battery
```

Response:
```json
{
  "type": "response",
  "status": "ok",
  "data": {
    "source": "BAT0",
    "percentage": 85.0,
    "state": "charging",
    "time_remaining_minutes": 120
  }
}
```

Protocol:
```json
{"type": "system.battery"}
```

## Idle Detection

Check how long the system has been idle:

```bash
deskbrid system idle
```

Response:
```json
{
  "type": "response",
  "status": "ok",
  "data": {
    "idle_ms": 300000
  }
}
```

Protocol:
```json
{"type": "system.idle"}
```

## Inhibit System

Prevent system sleep, screensaver, or session lock:

```bash
deskbrid system inhibit suspend --who "backup-script" --why "long-running backup"
```

Response:
```json
{
  "type": "response",
  "status": "ok",
  "data": {
    "inhibitor_id": 42
  }
}
```

Protocol:
```json
{
  "type": "system.inhibit",
  "what": "suspend",
  "who": "backup-script",
  "why": "long-running backup"
}
```

What to inhibit:
- `suspend` - Prevent system suspend
- `sleep` - Prevent system sleep
- `idle` - Prevent idle activation
- `logout` - Prevent automatic logout

### Release Inhibit

```bash
deskbrid system release-inhibit 42
```

Protocol:
```json
{"type": "system.release_inhibit", "inhibitor_id": 42}
```

## Session Management

### List Sessions

```bash
deskbrid system sessions
```

Response:
```json
{
  "type": "response",
  "status": "ok",
  "data": {
    "sessions": [
      {
        "session_id": "1",
        "username": "user",
        "seat": "seat0",
        "active": true
      }
    ]
  }
}
```

Protocol:
```json
{"type": "system.sessions"}
```

### Lock Session

```bash
deskbrid system lock-session
deskbrid system lock-session --session 2
```

Protocol:
```json
{"type": "system.lock_session"}
```

### Switch User

```bash
deskbrid system switch-user alice
```

Protocol:
```json
{"type": "system.switch_user", "username": "alice"}
```

## Privilege Escalation

### Check Auth

Check if an action requires authorization:

```bash
deskbrid system check-auth system.power
```

Response:
```json
{
  "type": "response",
  "status": "ok",
  "data": {
    "authorized": false,
    "action_id": "system.power"
  }
}
```

### Elevate Privileges

Request privilege elevation:

```bash
deskbrid system elevate system.power --reason "User requested shutdown"
```

Response:
```json
{
  "type": "response",
  "status": "ok",
  "data": {
    "authorized": true
  }
}
```

## Confinement Status

Check if running in a sandbox (Flatpak, Snap):

```bash
deskbrid system confinement
```

Response:
```json
{
  "type": "response",
  "status": "ok",
  "data": {
    "confined": true,
    "type": "flatpak",
    "sandbox": true
  }
}
```

## Python Example

```python
from deskbrid import Deskbrid

client = Deskbrid()

# Get system status
info = client.info()
print(f"Desktop: {info.desktop}, Session: {info.session_type}")

# Inhibit sleep during long operation
inhibit = client.inhibit_system("suspend", who="backup", why="backup running")
try:
    # ... long running task ...
    pass
finally:
    client.release_inhibit(inhibit["inhibitor_id"])
```