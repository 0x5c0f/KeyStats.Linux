# STATUS CHECKLIST

## Daemon Performance Optimizations

Branch: `fix/daemon-optimizations`
Spec: `docs/superpowers/specs/2026-05-29-daemon-optimizations-design.md`

### Optimization 1: Batch Key Count Flush

- [ ] Add `pending_keys: HashMap<String, u64>` field to `StatsManager`
- [ ] Update `new_with_conn` to initialize `pending_keys`
- [ ] `record_key_press`: accumulate to `pending_keys` instead of direct DB write
- [ ] `flush_to_db`: batch write `pending_keys` then clear
- [ ] `reset_today`: clear `pending_keys`
- [ ] `db/schema.rs`: add `batch_incr_key_counts` function
- [ ] Update test `key_press_increments_count`
- [ ] Add test `flush_writes_pending_keys`

### Optimization 3: Silent Error Logging

- [ ] `flush_to_db`: replace `.ok()` with warning log
- [ ] `delete_today_key_counts`: replace `.ok()` with warning log
- [ ] `delete_all`: replace `.ok()` with warning log
- [ ] Migration ALTER TABLE: keep `.ok()` (idempotent, expected failure)

### Optimization 2: Lock Error Handling

- [ ] Add `lock_stats` helper function
- [ ] `event_loop.rs`: replace 1 `lock().unwrap()`
- [ ] `dbus/service.rs`: replace 8 `lock().unwrap()` calls

### Verification

- [ ] `cargo build -p keystats-daemon` passes
- [ ] `cargo test -p keystats-daemon` passes
- [ ] `cargo clippy -p keystats-daemon` no new warnings
- [ ] Manual test: keystroke counting works, top_keys returns data
