# Daemon Performance Optimizations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate per-keystroke SQLite writes, harden error handling in daemon lock paths, and add logging for silently swallowed errors.

**Architecture:** Accumulate key counts in a `HashMap<String, u64>` in memory, flush to DB in a single transaction every 2 seconds alongside existing `daily_stats` upsert. Replace all `lock().unwrap()` with a helper that logs on poison. Replace `.ok()` with warning logs on DB errors.

**Tech Stack:** Rust, rusqlite, std::sync::Mutex, tracing

---

## File Map

| File | Changes |
|------|---------|
| `crates/keystats-daemon/src/db/schema.rs` | Add `batch_incr_key_counts()` function |
| `crates/keystats-daemon/src/stats/manager.rs` | Add `pending_keys` field, batch flush logic, `lock_stats` helper, error logging |
| `crates/keystats-daemon/src/input/event_loop.rs` | Use `lock_stats` instead of `lock().unwrap()` |
| `crates/keystats-daemon/src/dbus/service.rs` | Use `lock_stats` instead of `lock().unwrap()` |

---

### Task 1: Add `batch_incr_key_counts` to schema.rs

**Files:**
- Modify: `crates/keystats-daemon/src/db/schema.rs:160-167`

- [ ] **Step 1: Add `use std::collections::HashMap;` import**

At the top of `crates/keystats-daemon/src/db/schema.rs`, add:

```rust
use std::collections::HashMap;
```

- [ ] **Step 2: Add `batch_incr_key_counts` function after `incr_key_count`**

After the existing `incr_key_count` function (line 167), add:

```rust
pub fn batch_incr_key_counts(
    conn: &Connection,
    date: &str,
    counts: &HashMap<String, u64>,
) -> Result<(), rusqlite::Error> {
    let tx = conn.transaction()?;
    for (key_name, count) in counts {
        tx.execute(
            "INSERT INTO key_counts (date, key_name, count) VALUES (?1, ?2, ?3)
             ON CONFLICT(date, key_name) DO UPDATE SET count = count + ?3",
            rusqlite::params![date, key_name, count],
        )?;
    }
    tx.commit()
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p keystats-daemon`
Expected: compiles without errors

---

### Task 2: Add `pending_keys` field and batch flush to StatsManager

**Files:**
- Modify: `crates/keystats-daemon/src/stats/manager.rs`

- [ ] **Step 1: Add `pending_keys` field to `StatsManager` struct**

In `crates/keystats-daemon/src/stats/manager.rs`, add the field after `mouse_coalesce` (line 17):

```rust
#[allow(dead_code)]
pub struct StatsManager {
    db: rusqlite::Connection,
    today: String,
    stats: DailyStats,
    kps_tracker: RateTracker,
    cps_tracker: RateTracker,
    last_flush: Instant,
    flush_interval: std::time::Duration,
    mouse_coalesce: (f64, f64),
    pending_keys: std::collections::HashMap<String, u64>,
}
```

- [ ] **Step 2: Initialize `pending_keys` in `new_with_conn`**

In `new_with_conn` (line 36-46), add `pending_keys` to the struct initialization:

```rust
Ok(Self {
    db: conn,
    today,
    stats,
    kps_tracker: RateTracker::new(),
    cps_tracker: RateTracker::new(),
    last_flush: Instant::now(),
    flush_interval: std::time::Duration::from_secs(2),
    mouse_coalesce: (0.0, 0.0),
    pending_keys: std::collections::HashMap::new(),
})
```

- [ ] **Step 3: Update `record_key_press` to accumulate in memory**

Replace lines 69-78 with:

```rust
pub fn record_key_press(&mut self, key_name: &str, track_breakdown: bool) {
    self.check_midnight();
    self.stats.key_presses += 1;
    self.kps_tracker.record();
    self.update_peaks();
    if track_breakdown {
        *self.pending_keys.entry(key_name.to_string()).or_insert(0) += 1;
    }
    self.maybe_flush();
}
```

- [ ] **Step 4: Update `flush_to_db` to batch write pending keys**

Replace lines 123-126 with:

```rust
fn flush_to_db(&mut self) {
    self.stats.updated_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    if let Err(e) = db::schema::upsert_daily_stats(&self.db, &self.stats) {
        tracing::warn!("Failed to flush daily stats: {}", e);
    }
    if !self.pending_keys.is_empty() {
        if let Err(e) = db::schema::batch_incr_key_counts(&self.db, &self.today, &self.pending_keys) {
            tracing::warn!("Failed to flush key counts: {}", e);
        }
        self.pending_keys.clear();
    }
}
```

- [ ] **Step 5: Update `reset_today` to clear pending keys**

In `reset_today` (line 189-195), add `self.pending_keys.clear();` after `self.cps_tracker = RateTracker::new();`:

```rust
pub fn reset_today(&mut self) {
    self.stats = DailyStats::today();
    self.kps_tracker = RateTracker::new();
    self.cps_tracker = RateTracker::new();
    self.pending_keys.clear();
    if let Err(e) = db::schema::delete_today_key_counts(&self.db, &self.today) {
        tracing::warn!("Failed to delete key counts: {}", e);
    }
    self.flush_to_db();
}
```

- [ ] **Step 6: Update `clear_all_data` error logging**

Replace line 201-204:

```rust
pub fn clear_all_data(&mut self) {
    self.reset_today();
    if let Err(e) = db::schema::delete_all(&self.db) {
        tracing::warn!("Failed to clear all data: {}", e);
    }
}
```

- [ ] **Step 7: Verify it compiles**

Run: `cargo build -p keystats-daemon`
Expected: compiles without errors

---

### Task 3: Update existing tests for batch flush behavior

**Files:**
- Modify: `crates/keystats-daemon/src/stats/manager.rs:207-282`

- [ ] **Step 1: Update `key_press_increments_count` test**

The existing test at line 218-224 calls `record_key_press("A", true)` twice. With the new behavior, key counts are in `pending_keys` until flush. The `snapshot().key_presses` counter still increments immediately, so the existing assertion `assert_eq!(mgr.snapshot().key_presses, 2)` still passes.

No change needed to this test — `key_presses` is a direct field increment, not batched.

- [ ] **Step 2: Add test `flush_writes_pending_keys`**

After the `import_merge_adds_stats` test (line 282), add:

```rust
#[test]
fn flush_writes_pending_keys() {
    let mut mgr = test_mgr();
    mgr.record_key_press("A", true);
    mgr.record_key_press("A", true);
    mgr.record_key_press("B", true);
    // Force flush
    mgr.force_flush();

    let keys = mgr.top_keys(10).unwrap();
    assert_eq!(keys.len(), 2);
    let a = keys.iter().find(|k| k.key_name == "A").unwrap();
    assert_eq!(a.count, 2);
    let b = keys.iter().find(|k| k.key_name == "B").unwrap();
    assert_eq!(b.count, 1);
}

#[test]
fn reset_clears_pending_keys() {
    let mut mgr = test_mgr();
    mgr.record_key_press("A", true);
    mgr.record_key_press("B", true);
    assert_eq!(mgr.pending_keys.len(), 2);
    mgr.reset_today();
    assert!(mgr.pending_keys.is_empty());
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p keystats-daemon`
Expected: all tests pass (including new ones)

---

### Task 4: Add `lock_stats` helper to StatsManager

**Files:**
- Modify: `crates/keystats-daemon/src/stats/manager.rs`

- [ ] **Step 1: Add `lock_stats` helper function**

At the end of the `impl StatsManager` block (before `#[cfg(test)]`), or as a standalone function after the impl block, add:

```rust
/// Lock the shared StatsManager, returning None and logging on poison.
pub fn lock_stats(
    stats: &std::sync::Arc<std::sync::Mutex<StatsManager>>,
) -> Option<std::sync::MutexGuard<'_, StatsManager>> {
    match stats.lock() {
        Ok(guard) => Some(guard),
        Err(e) => {
            tracing::error!("StatsManager mutex poisoned: {}", e);
            None
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build -p keystats-daemon`
Expected: compiles without errors

---

### Task 5: Replace `lock().unwrap()` in event_loop.rs

**Files:**
- Modify: `crates/keystats-daemon/src/input/event_loop.rs:131`

- [ ] **Step 1: Add import for `lock_stats`**

At the top of `event_loop.rs`, update the import from `crate::stats::manager::StatsManager` to also import `lock_stats`:

```rust
use crate::stats::manager::{StatsManager, lock_stats};
```

- [ ] **Step 2: Replace the lock call**

Replace line 131:

```rust
// Before:
let mut mgr = stats.lock().unwrap();

// After:
let Some(mut mgr) = lock_stats(&stats) else { continue; };
```

The full block (lines 129-135) becomes:

```rust
let mut total = 0usize;
{
    let Some(mut mgr) = lock_stats(&stats) else { continue; };
    for device in &mut devices {
        total += process_device(device, &mut mgr);
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p keystats-daemon`
Expected: compiles without errors

---

### Task 6: Replace `lock().unwrap()` in dbus/service.rs

**Files:**
- Modify: `crates/keystats-daemon/src/dbus/service.rs` (8 instances)

- [ ] **Step 1: Add import for `lock_stats`**

At the top of `service.rs`, update the import:

```rust
use crate::stats::manager::{StatsManager, lock_stats};
```

- [ ] **Step 2: Replace in `get_today_stats` (line 48)**

```rust
fn get_today_stats(&self) -> HashMap<String, zbus::zvariant::Value<'static>> {
    let Some(mgr) = lock_stats(&self.stats) else { return HashMap::new(); };
    let s = mgr.snapshot();
    // ... rest unchanged
```

- [ ] **Step 3: Replace in `get_rates` (line 109)**

```rust
fn get_rates(&self) -> HashMap<String, zbus::zvariant::Value<'static>> {
    let Some(mgr) = lock_stats(&self.stats) else { return HashMap::new(); };
    let r = mgr.rates();
    // ... rest unchanged
```

- [ ] **Step 4: Replace in `get_history` (line 126)**

```rust
fn get_history(&self, days: u32) -> String {
    let Some(mgr) = lock_stats(&self.stats) else { return String::new(); };
    match mgr.history(days) {
        Ok(history) => serde_json::to_string(&history).unwrap_or_default(),
        Err(e) => format!(r#"{{"error":"{}"}}"#, e),
    }
}
```

- [ ] **Step 5: Replace in `get_top_keys` (line 139)**

```rust
fn get_top_keys(&self, limit: u32) -> String {
    let Some(mgr) = lock_stats(&self.stats) else { return String::new(); };
    match mgr.top_keys(limit) {
        Ok(keys) => serde_json::to_string(&keys).unwrap_or_default(),
        Err(e) => format!(r#"{{"error":"{}"}}"#, e),
    }
}
```

- [ ] **Step 6: Replace in `reset_today` (line 147)**

```rust
fn reset_today(&self) -> bool {
    let Some(mut mgr) = lock_stats(&self.stats) else { return false; };
    mgr.reset_today();
    true
}
```

- [ ] **Step 7: Replace in `clear_all_data` (line 152)**

```rust
fn clear_all_data(&self) -> bool {
    let Some(mut mgr) = lock_stats(&self.stats) else { return false; };
    mgr.clear_all_data();
    true
}
```

- [ ] **Step 8: Replace in `export_data` (line 158)**

```rust
fn export_data(&self) -> String {
    let Some(mgr) = lock_stats(&self.stats) else { return String::new(); };
    mgr.export_data().unwrap_or_default()
}
```

- [ ] **Step 9: Replace in `import_data` (line 168)**

```rust
fn import_data(&self, json: &str, mode: &str) -> bool {
    let import_mode = match mode {
        "overwrite" => ImportMode::Overwrite,
        _ => ImportMode::Merge,
    };
    let Some(mut mgr) = lock_stats(&self.stats) else { return false; };
    mgr.import_data(json, import_mode).is_ok()
}
```

- [ ] **Step 10: Verify it compiles**

Run: `cargo build -p keystats-daemon`
Expected: compiles without errors

---

### Task 7: Final verification

- [ ] **Step 1: Full build**

Run: `cargo build -p keystats-daemon -p keystatsctl`
Expected: compiles without errors

- [ ] **Step 2: Full test suite**

Run: `cargo test -p keystats-daemon`
Expected: all tests pass

- [ ] **Step 3: Clippy check**

Run: `cargo clippy -p keystats-daemon -- -D warnings`
Expected: no warnings

- [ ] **Step 4: Manual smoke test**

Run the daemon briefly to verify it starts and processes events:

```bash
cargo run -p keystats-daemon
# Press some keys, Ctrl+C after a few seconds
# Verify: no panics, no error logs
```
