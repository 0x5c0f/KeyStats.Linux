# Per-Application Key Statistics — Feasibility Research

Date: 2026-05-28
Status: Research paused — deferred to future iteration
Branch: `research/per-app-stats`

## Goal

Track keyboard/mouse statistics broken down by the focused application, so users can see which apps they use most intensively.

## Current Architecture Limitation

The evdev input pipeline (`crates/keystats-daemon/src/input/event_loop.rs`) reads `/dev/input/event*` device files. evdev events contain only:
- Event type (key press, relative axis, etc.)
- Key code / axis value
- Device identity (name, path)

evdev does **not** provide any window or application context. Getting the focused application requires querying the display server separately.

## Focused Window Detection — Platform Analysis

### X11 (Xlib)

- `XGetInputFocus()` → returns the focused X window
- `_NET_WM_PID` property on that window → owning PID
- `WM_CLASS` property → application class/instance name
- CLI equivalent: `xdotool getwindowpid $(xdotool getwindowfocus)`
- **Reliability:** High. Nearly all modern WMs set EWMH properties.
- **Performance:** Local X11 socket call, ~0.05ms per query.
- **Dependencies:** `x11` or `xlib` Rust crate.

### GNOME Wayland (org.gnome.Shell.Eval)

```bash
gdbus call --session --dest org.gnome.Shell \
  --object-path /org/gnome/Shell \
  --method org.gnome.Shell.Eval \
  "global.display.focus_window?.wm_class"
```

- Returns WM class name; can also get PID via `get_pid()`.
- **Reliability:** Works but is an **unstable internal API** — may break across GNOME versions.
- **Performance:** D-Bus session bus call, ~0.1–0.5ms per query.
- **Dependencies:** None (D-Bus is already used by the daemon).

### wlroots Wayland (Sway, Hyprland)

- `ext-foreign-toplevel-list-v1` (staging protocol in wayland-protocols)
- Exposes toplevels with `app_id`, `title`, and `focused` state.
- **Reliability:** Stable in wlroots compositors. Not available in GNOME or KDE.
- **Performance:** Event-driven (subscribe to state changes), near-zero per-event cost.
- **Dependencies:** `wayland-client` crate + protocol XML.

### KDE Wayland

- KDE has its own protocols for window management.
- Needs further investigation (deferred).

### Universal Fallback

- No single Linux API covers all compositors.
- Must detect display server at runtime (`$XDG_SESSION_TYPE`, `$WAYLAND_DISPLAY`, `XDG_CURRENT_DESKTOP`) and branch accordingly.

## Proposed Architecture (Draft)

### Runtime Detection

```
if XDG_SESSION_TYPE == "x11" → X11 path
elif GNOME_DESKTOP_SESSION_ID or XDG_CURRENT_DESKTOP contains "GNOME" → Eval path
elif SWAYSOCK or HYPRLAND_INSTANCE_SIGNATURE → wlroots path
else → no per-app tracking (graceful degradation)
```

### Performance Optimization

Polling the focused window on every key press is wasteful. Proposed strategy:

1. **Cache the focused app** — store `(app_id, pid, wm_class, timestamp)`.
2. **Refresh on interval** — re-query every 2–5 seconds, or on the next event after the cache expires.
3. **Refresh on focus change** — where possible (wlroots events, X11 PropertyNotify), use event-driven updates instead of polling.

This keeps per-event cost near zero while maintaining reasonable accuracy.

### Data Model Changes

**Option A: Extend `key_counts` table**

```sql
ALTER TABLE key_counts ADD COLUMN app_id TEXT NOT NULL DEFAULT '';
-- Composite PK becomes (date, key_name, app_id)
```

- Pros: Simple, minimal schema change.
- Cons: Table grows multiplicatively (keys × apps), query complexity increases.

**Option B: New `app_stats` table (recommended)**

```sql
CREATE TABLE app_stats (
    date       TEXT NOT NULL,
    app_id     TEXT NOT NULL,
    key_presses INTEGER NOT NULL DEFAULT 0,
    left_clicks INTEGER NOT NULL DEFAULT 0,
    right_clicks INTEGER NOT NULL DEFAULT 0,
    middle_clicks INTEGER NOT NULL DEFAULT 0,
    mouse_distance REAL NOT NULL DEFAULT 0,
    scroll_distance REAL NOT NULL DEFAULT 0,
    PRIMARY KEY (date, app_id)
);

CREATE TABLE app_key_counts (
    date      TEXT NOT NULL,
    app_id    TEXT NOT NULL,
    key_name  TEXT NOT NULL,
    count     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (date, app_id, key_name)
);
```

- Pros: Clean separation, queries are straightforward, doesn't bloat existing tables.
- Cons: More tables, migration needed.

### StatsManager Changes

- `record_key_press(key_name, track_breakdown)` → `record_key_press(key_name, app_id, track_breakdown)`
- `record_click(role)` → `record_click(role, app_id)`
- New `AppFocusTracker` struct that handles caching and platform-specific queries.
- The event loop passes `app_id` along with each event to the stats manager.

### D-Bus API Additions

| Method | Return | Description |
|--------|--------|-------------|
| `GetTopApps(limit: u32)` | JSON string | Top apps by key press count today |
| `GetAppStats(app_id: s)` | JSON string | Full stats for a specific app |
| `GetAppKeyBreakdown(app_id: s, limit: u32)` | JSON string | Top keys for a specific app |

### GNOME Extension UI

- New "Applications" section in the popup dashboard.
- List top N apps with key press + click counts.
- Clicking an app could show its key breakdown.

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| GNOME Eval API breaks in future versions | Per-app stats stop working on GNOME Wayland | Graceful degradation, monitor GNOME releases |
| D-Bus query overhead on every key press | Performance regression | Cache + interval refresh strategy |
| Schema migration for existing users | Data loss or migration failure | Version-gated migration with backup |
| KDE/Wayland compositor fragmentation | Incomplete coverage | Start with X11 + GNOME, expand incrementally |

## Implementation Phases (Proposed)

1. **Phase 1 — X11 support:** Implement Xlib-based focus detection. Lowest risk, highest reliability.
2. **Phase 2 — GNOME Wayland:** Add Eval-based detection as fallback. Test across GNOME versions.
3. **Phase 3 — wlroots:** Add `ext-foreign-toplevel-list` support for Sway/Hyprland users.
4. **Phase 4 — KDE:** Investigate and implement KDE-specific protocols.
5. **Phase 5 — UI:** GNOME extension dashboard integration.

## Dependencies to Evaluate

- `x11` or `xlib` Rust crate for X11 support
- `wayland-client` crate for wlroots protocols
- `zbus` (already used) for GNOME Eval D-Bus calls

## Open Questions

- Should per-app tracking be opt-in (privacy concern: reveals which apps are in use)?
- How to handle apps with no WM_CLASS or generic names?
- Should the GNOME extension show per-app stats, or is the D-Bus API sufficient for CLI/third-party consumers?
- Storage retention: should per-app data be retained longer than per-day aggregates?

## References

- [wayland-protocols ext-foreign-toplevel-list-v1](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/staging/ext-foreign-toplevel-list/ext-foreign-toplevel-list-v1.xml)
- [EWMH Specification](https://specifications.freedesktop.org/wm-spec/wm-spec-latest.html)
- [ActivityWatch aw-watcher-window](https://github.com/ActivityWatch/aw-watcher-window) — existing per-app tracking on Linux
