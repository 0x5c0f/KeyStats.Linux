# KeyStats.Linux — Domain Context

## What is KeyStats

KeyStats is a Linux desktop application that tracks keyboard and mouse usage statistics in real-time. It runs as a background daemon, reads input events from `/dev/input` devices, and stores daily aggregated stats in SQLite.

## Architecture

```
┌─────────────────┐    D-Bus     ┌──────────────────┐
│  GNOME Shell    │◄────────────►│  keystats-daemon  │
│  Extension      │              │  (background)     │
└─────────────────┘              └──────────────────┘
                                        │
                                   D-Bus signals
                                        │
                                        ▼
                                 ┌──────────────────┐
                                 │ keystats-overlay  │
                                 │ (optional, new)   │
                                 └──────────────────┘
```

### Components

- **keystats-core**: Shared data models and utilities
- **keystats-daemon**: Background service (evdev → SQLite → D-Bus)
- **keystatsctl**: CLI tool for querying stats
- **GNOME Shell Extension**: Panel indicator showing live stats
- **keystats-overlay** (new): Standalone keystroke visualization overlay

### IPC

- **Daemon ↔ Extension**: D-Bus methods (`GetTodayStats`, `GetRates`, etc.)
- **Daemon → Overlay**: D-Bus signals (`KeyPressed`, `KeyReleased`)

## Key Terms

| Term | Meaning |
|------|---------|
| **DailyStats** | Per-day aggregated counters (key presses, clicks, distances) |
| **RatesSnapshot** | Instantaneous KPS/CPS rates |
| **InputEvent** | Real-time input event (key press/release) for D-Bus signal |
| **evdev** | Linux input event interface (`/dev/input/event*`) |
| **override_redirect** | X11 window attribute that bypasses the window manager |
| **layer-shell** | Wayland protocol for creating overlay/notification windows |

## Design Decisions

See `docs/adr/` for architectural decision records.
