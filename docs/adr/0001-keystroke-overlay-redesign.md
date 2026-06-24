# ADR-0001: Keystroke Overlay Redesign

## Status

Proposed

## Context

The previous keystroke overlay implementation (`feature/keystroke-overlay`) had critical bugs across all categories:

1. **Display issues**: Key loss, display delay, incorrect Shift modifier handling
2. **Window management**: White flash on startup, position drift, taskbar entry, WM interference
3. **Performance**: High CPU usage from polling loops
4. **Compatibility**: X11-only, no Wayland support

The overlay's core purpose is real-time keystroke display (OBS-style) for screencasting, streaming, and accessibility.

## Decision

Redesign the overlay from scratch with the following architecture:

### IPC: D-Bus Signal

The daemon emits `KeyPressed` and `KeyReleased` signals on the existing D-Bus interface (`io.github.x0x5c0f.KeyStats1`). The overlay is a separate process that subscribes to these signals.

**Rationale**: D-Bus is already the IPC layer between daemon and GNOME extension. Adding signals is minimal change. The overlay as a separate process can be started/stopped independently.

### Daemon Changes

The event loop must emit both press and release events:

- `InputEvent::KeyPress { name: String }` — key pressed
- `InputEvent::KeyRelease { name: String }` — key released (new)

Modifier state (Ctrl, Shift, Alt, Super) is tracked in the daemon. When a non-modifier key is pressed while modifiers are held, the signal carries the combined name (e.g. `"Ctrl+C"`).

### Overlay Process: GTK4 + Conditional Layer-Shell

- **X11**: GTK4 window with `override_redirect` (bypasses WM entirely — no animation, no taskbar, no focus stealing)
- **Wayland**: GTK4 + `gtk4-layer-shell` using ext-layer-shell protocol (GNOME 47+)
- Runtime detection: `gdk_display_get_type()` → `GDK_TYPE_X11_DISPLAY` or `GDK_TYPE_WAYLAND_DISPLAY`

### Visual Design

- **Theme**: Dark background (`rgba(0,0,0,0.7)`), white text, rounded corners
- **Layout**: Vertical stack, dynamic expansion (new keys push old ones down)
- **Fade**: Key appears on press, starts fading on release (opacity 1.0 → 0.0 over ~300ms)
- **Position**: Configurable corner (top-left default), percentage-based margin

### Configuration

CLI arguments:
- `--position <corner>` — top-left, top-right, bottom-left, bottom-right
- `--max-keys <n>` — maximum visible keys (default: 10)
- `--fade-duration <ms>` — fade-out duration in milliseconds (default: 300)
- `--font-size <px>` — font size (default: 16)
- `--margin <percent>` — margin as screen percentage (default: 5)

## Consequences

### Positive
- Clean separation: overlay is an independent process, no code coupling with daemon
- D-Bus signals are reliable and well-supported on both X11 and Wayland
- GTK4 + layer-shell provides proper overlay behavior on both display protocols
- Fade-on-release gives natural visual feedback

### Negative
- D-Bus signal latency (~1-5ms) is acceptable for visual display but not for input recording
- gtk4-layer-shell requires GNOME 47+ for full Wayland support
- X11 override_redirect requires unsafe code for Xlib calls

### Risks
- GNOME Wayland may not support ext-layer-shell on older versions → fallback to regular window with keep_above
- Modifier state tracking in daemon adds complexity → must handle edge cases (modifier tap, multiple modifiers)
