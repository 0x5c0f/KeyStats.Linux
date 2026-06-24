# Audit Log

Per-project audit history for KeyStats.Linux.

---

## 2026-06-24 — First Audit

| Date | Project | Mode | 🔴 | 🟠 | 🟡 | 🔵 | Health | Notes |
|------|---------|------|-----|-----|-----|-----|--------|-------|
| 2026-06-24 | KeyStats.Linux | Multi-Crate | 0 | 14 | 0 | 1 | 🟡 | First audit, cargo-audit not installed |

### Findings

| ID | Severity | Summary | Location |
|----|----------|---------|----------|
| C2 | High | expect() in D-Bus connection setup | `crates/keystats-daemon/src/dbus/service.rs:37-43` |
| C2 | High | expect() for GTK Display access | `crates/keystats-overlay/src/overlay.rs:187,209,263,422` |
| C2 | High | expect() on StatsManager init | `crates/keystats-daemon/src/main.rs:28` |
| C2 | High | expect() on D-Bus interface name | `crates/keystats-overlay/src/dbus_client.rs:37` |
| S1 | High | Undocumented unsafe X11 FFI | `crates/keystats-overlay/src/overlay.rs:326-352` |
| S1 | High | Undocumented unsafe X11 FFI | `crates/keystats-overlay/src/overlay.rs:478-490` |
| C10 | Low | TODO marker | `crates/keystatsctl/src/commands.rs:91` |

### Gotcha Validation

No new gotcha patterns discovered in this audit.

### Notes

- Project uses multi-crate workspace with 4 crates (keystats-core, keystats-daemon, keystatsctl, keystats-overlay)
- Does not follow standard 7-crate layout — acceptable for system utility
- D-Bus used for IPC between daemon and overlay
- GTK4 overlay supports X11 and Wayland via feature flags
- cargo-audit not installed — security scan incomplete

---

## 2026-06-24 — Second Audit (Post-Fix)

| Date | Project | Mode | 🔴 | 🟠 | 🟡 | 🔵 | Health | Notes |
|------|---------|------|-----|-----|-----|-----|--------|-------|
| 2026-06-24 | KeyStats.Linux | Multi-Crate | 0 | 2 | 0 | 0 | 🟡 | All C2/C10 issues resolved, S1 now documented |

### Findings

| ID | Severity | Summary | Location | Status |
|----|----------|---------|----------|--------|
| S1 | High | Documented unsafe X11 FFI | `crates/keystats-overlay/src/overlay.rs:355` | ✅ Resolved (docs added) |
| S1 | High | Documented unsafe X11 FFI | `crates/keystats-overlay/src/overlay.rs:517` | ✅ Resolved (docs added) |

### Resolved Issues (from First Audit)

| ID | Summary | Resolution |
|----|---------|------------|
| C2 | expect() in D-Bus service | Replaced with `and_then` chain + `match` |
| C2 | expect() for GTK Display | Replaced with `let Some(display) = ... else` |
| C2 | expect() on StatsManager init | Replaced with `match` + `process::exit(1)` |
| C2 | expect() on D-Bus interface | Replaced with `match` statements |
| C10 | TODO marker | Extracted `MAX_EVENT_DEVICES` constant |

### Gotcha Validation

**New pattern discovered:** The audit script flags `unsafe {` blocks regardless of whether they have SAFETY documentation. This is by design — the script cannot determine if the documentation is adequate. Manual review is required to confirm the safety invariants are properly documented.

**Recommendation:** Consider updating the script to recognize `// SAFETY:` comments and mark those as "documented" vs "undocumented" in the output.

### Notes

- Excellent fix quality — all 14 issues resolved within the same day
- Safety documentation on unsafe blocks is thorough and follows Rust best practices
- Error handling patterns are consistent across the codebase
- cargo-audit still not installed — security scan incomplete

---

*Next audit: Compare findings against this entry. Mark recurring/new/resolved issues.*
