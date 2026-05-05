# deskbrid

**The HAL your Linux desktop agents are missing.**

A standalone daemon that wraps Wayland protocols, DBus APIs, and PipeWire into a single JSON-over-Unix-socket protocol. Any agent platform — Hermes, Praxis, Claude Code, OpenAI Operator, whatever comes next — can connect and get full desktop control.

```json
→ {"action": "window:focus", "params": {"app_id": "firefox"}}
→ {"action": "inject:type", "params": {"text": "git push origin main\n"}}
← {"type": "event", "event": "clipboard", "data": {"text": "git push origin main"}}
```

## Why

macOS agents walk on water because they've got AppleScript, Accessibility APIs, and a platform that doesn't fight back. Linux agents have `xdotool` that breaks on Wayland and a pile of different DBus APIs with different conventions.

Deskbrid is the **one thing to install** that makes your agent native on Linux desktop — window focus tracking, clipboard access, input injection, screen capture, notification monitoring, audio control. A standard protocol any agent implements against.

## Quick Start

```bash
# Install
cargo install deskbrid

# Run
deskbrid

# Test from another terminal
nc -U "$XDG_RUNTIME_DIR/deskbrid/socket"
{"type":"subscribe","events":["window:focus","clipboard","notifications"]}
```

## Protocol

See [PROTOCOL.md](PROTOCOL.md) for the full spec.

### Events (daemon → agent)
- `window:focus` / `window:open` / `window:close` — window lifecycle
- `clipboard` — clipboard content changes
- `notifications` — desktop notification arrival
- `idle` — user idle state changes
- `audio:node` — audio node state changes

### Actions (agent → daemon)
- `window:list` / `window:focus` — window management
- `inject:type` / `inject:key` / `inject:mouse` — input injection
- `clipboard:read` / `clipboard:write` — clipboard access
- `screenshot` / `screencast:start|stop` — screen capture
- `notification:send` — desktop notifications
- `display:list` — monitor configuration

## Client Libraries

- **Python**: `pip install deskbrid` *(planned)*
- **Rust**: `deskbrid = "0.1"` *(planned)*
- **TypeScript**: `npm install deskbrid` *(planned)*

## Architecture

```
┌─────────────────────┐
│    Agent (any)       │
│  JSON over Unix sock │
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│    deskbrid daemon    │
│  ┌──────┐ ┌───────┐  │
│  │ DBus │ │  Way- │  │
│  │ Hub  │ │ land  │  │
│  └──┬───┘ │  Ext  │  │
│     │     └───┬───┘  │
│  ┌──▼─────────▼───┐  │
│  │   PipeWire      │  │
│  │   Stream Mgr    │  │
│  └─────────────────┘  │
└────────────────────────┘
```

## Platform Support

| Desktop | Session | Status |
|---|---|---|
| GNOME 42+ | Wayland | ✅ Primary target |
| GNOME 40+ | Wayland | ⚠️ Untested, should work |
| KDE Plasma | Wayland | 🔄 Planned |
| Sway / wlroots | Wayland | 🔄 Planned |
| X11 (any) | X11 | ❌ Not planned (use xdotool/yad) |

## Why Standalone

Deskbrid is **not** tied to any agent platform. It's a Unix daemon with a documented protocol, like `pipewire` or `systemd`. If you're building an agent platform and you want desktop access, you implement the 15-line client for the protocol. Deskbrid does the rest.

> "But I can just call DBus from my agent."
>
> Cool. You'll need to learn GNOME Shell's Eval API, Mutter's RemoteDesktop session lifecycle, PipeWire's stream negotiation, the portal API for screenshots, and the notification spec. You'll need to figure out GVariant parsing. You'll need to handle permission grants and session teardown. OR you install one binary and send JSON.

## License

MIT
