# Linux / GNOME 版 KeyStats Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Linux version of KeyStats using a native daemon for evdev collection and a GNOME Shell extension for top-bar UI.

**Architecture:** `keystats-daemon` reads Linux evdev input events, aggregates privacy-preserving counts, persists to SQLite, and exposes a session D-Bus API. A GNOME Shell extension consumes that API for panel display and popup UI. The extension never reads `/dev/input/event*` directly.

**Tech Stack:** Rust, evdev, udev, zbus, rusqlite, systemd --user, GNOME Shell extension (GJS), GSettings.

**Spec:** `docs/superpowers/specs/2026-05-25-linux-gnome-design.md`

**Primary target:** GNOME 45+ on Ubuntu / Fedora. Linux packaging starts with `.deb`, `.rpm`, and a manual tarball.

**Accepted default decisions (2026-05-25):**
- Target Ubuntu GNOME + Fedora GNOME first, GNOME Shell 45+.
- Use Rust for daemon / CLI and GJS for GNOME Shell extension.
- Use normal user daemon with `input` group or dedicated `keystats` group + udev rule; no root resident daemon for MVP.
- MVP excludes app-by-app stats.
- Start implementation with Phase 0 spikes and Phase 1 workspace bootstrap.

---

## File Structure

### New files

| Path | Responsibility |
|---|---|
| `KeyStats.Linux/Cargo.toml` | Rust workspace. |
| `KeyStats.Linux/crates/keystats-core/` | Shared models, formatting, import/export payloads. |
| `KeyStats.Linux/crates/keystats-daemon/` | evdev reader, stats manager, SQLite persistence, D-Bus service. |
| `KeyStats.Linux/crates/keystatsctl/` | CLI for status, doctor, export, import, reset. |
| `KeyStats.Linux/packaging/systemd/keystats.service` | systemd user service unit. |
| `KeyStats.Linux/packaging/udev/60-keystats-input.rules` | optional udev permission rule. |
| `KeyStats.GNOME/keystats@debugtheworldbot.github.io/` | GNOME Shell extension source. |

### Existing files likely modified later

| Path | Modification |
|---|---|
| `README.md` / `README_ZH.md` | Add Linux install/use docs after MVP works. |
| `.github/workflows/release.yml` | Add Linux build/package jobs after local packaging stabilizes. |
| `docs/superpowers/specs/2026-05-25-linux-gnome-design.md` | Update decisions from spikes. |

---

## Verification Gates

- **Rust unit gate:** `cargo test --workspace`
- **Rust lint gate:** `cargo clippy --workspace --all-targets -- -D warnings`
- **Format gate:** `cargo fmt --all --check`
- **Daemon smoke gate:** `keystatsctl status` returns daemon status and today stats.
- **Permission smoke gate:** `keystatsctl doctor` clearly reports readable and blocked input devices.
- **GNOME smoke gate:** extension can enable, display counts, open popup, disable cleanly.
- **Privacy gate:** no code path persists raw event sequence, text, or mouse absolute trajectory.

---

# Phase 0 - Spike & Decide

Goal: validate the risky platform assumptions before committing to the full implementation.

### Task 0.1: evdev read spike

- [ ] On Ubuntu GNOME, list `/dev/input/event*` devices and capabilities.
- [ ] On Fedora GNOME, list `/dev/input/event*` devices and capabilities.
- [ ] Build a scratch Rust program using `evdev` to print only event type/code/value.
- [ ] Confirm keyboard `EV_KEY value=1`, autorepeat `value=2`, mouse buttons, `REL_X/Y`, `REL_WHEEL`.
- [ ] Record which permissions were required on each distro.

### Task 0.2: permission model spike

- [ ] Test normal user without extra group.
- [ ] Test user in `input` group.
- [ ] Test dedicated `keystats` group with udev rule.
- [ ] Test logout/login requirement after group change.
- [ ] Decide MVP installer recommendation and update the spec.

### Task 0.3: D-Bus spike

- [ ] Create a scratch `zbus` session service with `GetTodayStats()`.
- [ ] Call it from `gdbus`.
- [ ] Call it from a minimal GJS script.
- [ ] Confirm async calls do not block GNOME Shell.

### Task 0.4: GNOME extension spike

- [ ] Create minimal `metadata.json` and `extension.js`.
- [ ] Add top-bar indicator.
- [ ] Poll D-Bus every 1 second.
- [ ] Enable / disable extension repeatedly and verify cleanup.
- [ ] Test GNOME 45 and one newer version if available.

---

# Phase 1 - Rust Workspace Bootstrap

Goal: create the Linux project skeleton with no input permissions needed yet.

### Task 1.1: create workspace

- [ ] Add `KeyStats.Linux/Cargo.toml`.
- [ ] Add `keystats-core`, `keystats-daemon`, and `keystatsctl` crates.
- [ ] Add shared model structs: `DailyStats`, `RatesSnapshot`, `PermissionStatus`, `Settings`.
- [ ] Add JSON serialization for import/export compatibility.
- [ ] Add unit tests for defaults, total clicks, correction rate, and formatters.

### Task 1.2: add CLI baseline

- [ ] Implement `keystatsctl --version`.
- [ ] Implement `keystatsctl status` with stub output.
- [ ] Implement `keystatsctl doctor` with stub permission output.
- [ ] Add README section for local development commands.

### Task 1.3: CI-local verification

- [ ] Run `cargo fmt --all --check`.
- [ ] Run `cargo test --workspace`.
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`.

---

# Phase 2 - Storage and Stats Core

Goal: implement privacy-preserving aggregation independent of evdev.

### Task 2.1: SQLite schema

- [ ] Add `daily_stats`, `key_counts`, and `metadata` tables.
- [ ] Implement migrations with schema version.
- [ ] Store DB under `$XDG_STATE_HOME/keystats/stats.sqlite3`.
- [ ] Use WAL mode.

### Task 2.2: stats manager

- [ ] Implement `record_key_press(key_name)`.
- [ ] Implement `record_click(button_role)`.
- [ ] Implement `add_mouse_distance(distance)`.
- [ ] Implement `add_scroll_distance(distance)`.
- [ ] Implement KPS/CPS sliding windows.
- [ ] Implement midnight reset based on local date.
- [ ] Implement flush debounce and shutdown flush.

### Task 2.3: import/export

- [ ] Define Linux export payload version 1.
- [ ] Include current day and history.
- [ ] Implement overwrite import.
- [ ] Implement merge import with safe addition.
- [ ] Add tests for malformed, empty, overwrite, and merge cases.

---

# Phase 3 - evdev Input Pipeline

Goal: convert Linux input events into the stats manager calls.

### Task 3.1: device discovery

- [ ] Enumerate `/dev/input/event*`.
- [ ] Filter devices by capabilities.
- [ ] Identify keyboard-like devices.
- [ ] Identify pointer-like devices.
- [ ] Ignore devices that cannot contribute to supported metrics.

### Task 3.2: event loop

- [ ] Read multiple devices concurrently.
- [ ] Handle hotplug via udev monitor.
- [ ] Handle device removal without crashing.
- [ ] Coalesce `REL_X/Y` per `SYN_REPORT`.
- [ ] Ignore autorepeat by default.

### Task 3.3: key and button mapping

- [ ] Map Linux key codes to stable display names.
- [ ] Map `BTN_LEFT`, `BTN_RIGHT`, `BTN_SIDE`, `BTN_EXTRA`.
- [ ] Add tests for key names and button roles.
- [ ] Preserve privacy rule: do not store raw event order.

### Task 3.4: permission diagnostics

- [ ] Implement readable/blocked device scan.
- [ ] Report current groups.
- [ ] Suggest `input` group or `keystats` udev rule when needed.
- [ ] Expose diagnostics through `keystatsctl doctor`.

---

# Phase 4 - D-Bus Service

Goal: expose a stable UI API.

### Task 4.1: service object

- [ ] Register session bus name `io.github.debugtheworldbot.KeyStats`.
- [ ] Register object `/io/github/debugtheworldbot/KeyStats`.
- [ ] Implement interface `io.github.debugtheworldbot.KeyStats1`.

### Task 4.2: methods

- [ ] Implement `GetTodayStats`.
- [ ] Implement `GetRates`.
- [ ] Implement `GetHistory`.
- [ ] Implement `GetPermissionStatus`.
- [ ] Implement `ResetToday`.
- [ ] Implement `ExportData`.
- [ ] Implement `ImportData`.

### Task 4.3: signals

- [ ] Emit `StatsChanged` after debounced stats update.
- [ ] Emit `PermissionChanged` when readable device state changes.
- [ ] Emit `SettingsChanged` after settings update.
- [ ] Add `gdbus` manual test commands to README.

---

# Phase 5 - systemd User Service and Packaging Skeleton

Goal: make the daemon installable and restartable.

### Task 5.1: systemd service

- [ ] Add `packaging/systemd/keystats.service`.
- [ ] Set `ExecStart=/usr/bin/keystats-daemon`.
- [ ] Set `Restart=on-failure`.
- [ ] Use journal logging.
- [ ] Document dev install with `systemctl --user link`.

### Task 5.2: udev rule

- [ ] Add optional `60-keystats-input.rules`.
- [ ] Use `GROUP="keystats"` and `MODE="0640"`.
- [ ] Document security implications.
- [ ] Add `keystatsctl doctor` output that references this rule.

### Task 5.3: package scripts

- [ ] Add tarball layout script.
- [ ] Add `.deb` packaging draft.
- [ ] Add `.rpm` packaging draft.
- [ ] Do not wire GitHub release until MVP smoke tests pass.

---

# Phase 6 - GNOME Shell Extension MVP

Goal: top-bar UI using D-Bus only.

### Task 6.1: extension skeleton

- [ ] Add `metadata.json`.
- [ ] Add `extension.js` exporting an Extension subclass.
- [ ] Add `stylesheet.css`.
- [ ] Add enable/disable cleanup.
- [ ] Test install with `gnome-extensions install`.

### Task 6.2: D-Bus client

- [ ] Create async proxy for `io.github.debugtheworldbot.KeyStats1`.
- [ ] Read `GetTodayStats`.
- [ ] Subscribe to `StatsChanged`.
- [ ] Fall back to timed refresh if signal is unavailable.
- [ ] Show service unavailable state.

### Task 6.3: panel UI

- [ ] Add KeyStats icon or text indicator.
- [ ] Display key and click counts.
- [ ] Support compact number formatting.
- [ ] Avoid layout shifts when numbers grow.
- [ ] Ensure disable removes panel item.

### Task 6.4: popup UI

- [ ] Show today stats.
- [ ] Show KPS/CPS and peak values.
- [ ] Show permission status when blocked.
- [ ] Add Reset action through D-Bus.
- [ ] Add Settings action.

### Task 6.5: preferences

- [ ] Add GSettings schema.
- [ ] Add `prefs.js`.
- [ ] Add toggles for keys/clicks visibility.
- [ ] Add refresh interval option.
- [ ] Add dynamic color option as future-ready preference.

---

# Phase 7 - End-to-End MVP

Goal: one developer can install and use the Linux/GNOME build locally.

### Task 7.1: local install flow

- [ ] Build daemon release binary.
- [ ] Install daemon and CLI to a temp prefix or `/usr/local/bin`.
- [ ] Install systemd user service.
- [ ] Install GNOME extension.
- [ ] Run `keystatsctl doctor`.

### Task 7.2: manual smoke test

- [ ] Type keys and verify top-bar count increments.
- [ ] Click left/right/side buttons and verify count increments.
- [ ] Move mouse and verify distance increments.
- [ ] Scroll and verify scroll distance increments.
- [ ] Restart daemon and verify persisted data remains.
- [ ] Disable/enable extension and verify daemon keeps running.

### Task 7.3: privacy audit

- [ ] Search code for raw event logging.
- [ ] Confirm no text content is stored.
- [ ] Confirm no absolute mouse path is stored.
- [ ] Confirm DB contains only aggregate data.
- [ ] Update README privacy language.

---

# Phase 8 - Release Preparation

Goal: prepare for public beta.

### Task 8.1: documentation

- [ ] Add Linux section to `README.md`.
- [ ] Add Linux section to `README_ZH.md`.
- [ ] Add troubleshooting for permissions.
- [ ] Add uninstall instructions.

### Task 8.2: CI

- [ ] Add Linux Rust build job.
- [ ] Add `cargo fmt`, `cargo test`, `cargo clippy`.
- [ ] Add artifact build for tarball.
- [ ] Add package build only after packaging scripts are stable.

### Task 8.3: beta criteria

- [ ] Ubuntu GNOME smoke test passes.
- [ ] Fedora GNOME smoke test passes.
- [ ] Permission failure flow is understandable.
- [ ] No known raw input privacy violation.
- [ ] Known limitations documented: app stats unavailable, GNOME-first support.

---

## Open Questions

- Should Linux import/export be byte-compatible with macOS/Windows payloads or use a Linux-specific payload with migration tooling?
- Should the first public build ship GNOME extension system-wide or ask users to install it separately?
- Should mouse distance use raw device counts or attempt DPI-aware normalization?

## Resolved Questions

- **Initial target platforms:** Ubuntu GNOME + Fedora GNOME, GNOME Shell 45+.
- **Initial technology stack:** Rust daemon / CLI + GJS GNOME Shell extension.
- **Initial permission model:** normal user daemon with `input` group or dedicated `keystats` group + udev rule; no root resident daemon for MVP.
- **Initial feature boundary:** no app-by-app stats in MVP.
