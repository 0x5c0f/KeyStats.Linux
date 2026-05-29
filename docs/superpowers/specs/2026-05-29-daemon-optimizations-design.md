# Daemon Performance Optimizations

Date: 2026-05-29
Branch: `fix/daemon-optimizations`
Scope: `keystats-daemon` crate only

## Background

Based on project audit, three targeted optimizations were identified as high-value, low-risk improvements to the daemon's runtime behavior.

## Optimization 1: Batch Key Count Flush

### Problem

`record_key_press` calls `db::schema::incr_key_count()` on every keystroke. At 5-10 keys/sec, this fires an individual SQLite INSERT per key, causing unnecessary write pressure.

### Solution

Add `pending_keys: HashMap<String, u64>` to `StatsManager`. `record_key_press` accumulates counts in memory. `flush_to_db` writes all pending key counts in a single transaction alongside the existing `upsert_daily_stats`.

### Changes

**`stats/manager.rs`**:
- Add `pending_keys: HashMap<String, u64>` field to `StatsManager`
- `record_key_press`: replace `db::schema::incr_key_count()` with `*self.pending_keys.entry(...).or_insert(0) += 1`
- `flush_to_db`: call `db::schema::batch_incr_key_counts()` then `self.pending_keys.clear()`
- `reset_today`: also clear `pending_keys`
- Update `new_with_conn` to initialize `pending_keys: HashMap::new()`

**`db/schema.rs`**:
- Add `pub fn batch_incr_key_counts(conn, date, counts: &HashMap<String, u64>)`
  - Single transaction with loop over counts
  - Uses existing `INSERT ... ON CONFLICT DO UPDATE SET count = count + ?3` pattern

**Tests**:
- Update `key_press_increments_count` to verify `pending_keys` is populated
- Add `flush_writes_pending_keys` test to verify batch write works
- Existing `import_*` and `reset_*` tests continue to pass

## Optimization 2: Lock Error Handling

### Problem

9 instances of `stats.lock().unwrap()` across event loop and D-Bus service. If a thread panics while holding the mutex, the lock becomes poisoned and all subsequent `.unwrap()` calls propagate the panic, crashing the daemon.

### Solution

Define a helper function that returns `Option<MutexGuard>` with error logging.

### Changes

**`stats/manager.rs`** (or new `util.rs`):
```rust
pub fn lock_stats(stats: &Arc<Mutex<StatsManager>>) -> Option<MutexGuard<'_, StatsManager>> {
    match stats.lock() {
        Ok(guard) => Some(guard),
        Err(e) => {
            tracing::error!("StatsManager mutex poisoned: {}", e);
            None
        }
    }
}
```

**`input/event_loop.rs`** (1 instance):
- `stats.lock().unwrap()` -> `let Some(mut mgr) = lock_stats(&stats) else { continue; };`

**`dbus/service.rs`** (8 instances):
- Each `self.stats.lock().unwrap()` -> `let Some(mgr) = lock_stats(&self.stats) else { return <empty>; };`
- Return types: `HashMap` -> empty map, `String` -> empty string or error JSON, `bool` -> false

## Optimization 3: Silent Error Logging

### Problem

5 instances of `.ok()` silently discard SQLite errors. Data loss goes undetected.

### Solution

Replace `.ok()` with `.inspect_err(|e| tracing::warn!(...))` or `if let Err(e) = ... { tracing::warn!(...) }`.

### Changes

**`stats/manager.rs`**:
- `flush_to_db` line 125: `.ok()` -> log warning (this becomes the batch flush from Optimization 1)
- `delete_today_key_counts` line 193: `.ok()` -> log warning
- `delete_all` line 203: `.ok()` -> log warning

**`db/schema.rs`**:
- Migration ALTER TABLE line 48: **keep `.ok()`** — this is an idempotent migration step where failure (column already exists) is expected

## Implementation Order

1. Optimization 1 + 3 together (flush logic is being rewritten anyway)
2. Optimization 2 (independent, applied last)

## Non-Goals

- Polling model change (current 8ms sleep is adequate for background daemon)
- Channel architecture (premature for current scale)
- chrono -> time migration (separate cleanup, not performance-related)
- RwLock (event loop needs `&mut`, write lock would dominate)
