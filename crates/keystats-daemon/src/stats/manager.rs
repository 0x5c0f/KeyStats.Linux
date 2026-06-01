use std::time::Instant;

use keystats_core::{DailyStats, RatesSnapshot};

use super::rates::RateTracker;
use crate::db;

#[allow(dead_code)]
pub struct StatsManager {
    db: rusqlite::Connection,
    today: String,
    stats: DailyStats,
    kps_tracker: RateTracker,
    cps_tracker: RateTracker,
    last_flush: Instant,
    flush_interval: std::time::Duration,
    mouse_coalesce: (f64, f64), // pending dx, dy
    pending_keys: std::collections::HashMap<String, u64>,
}

#[allow(dead_code)]
impl StatsManager {
    pub fn new() -> Result<Self, rusqlite::Error> {
        let db = db::open()?;
        Self::new_with_conn(db)
    }

    pub fn new_with_conn(conn: rusqlite::Connection) -> Result<Self, rusqlite::Error> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let stats = db::schema::load_daily_stats(&conn, &today)?.unwrap_or_else(|| DailyStats {
            date: today.clone(),
            updated_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            ..Default::default()
        });

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
    }

    // --- Recording ---

    fn check_midnight(&mut self) {
        let current = chrono::Local::now().format("%Y-%m-%d").to_string();
        if current != self.today {
            // Flush old day before resetting
            self.flush_to_db();
            self.today = current;
            self.stats = DailyStats::today();
            self.kps_tracker = RateTracker::new();
            self.cps_tracker = RateTracker::new();
        }
    }

    fn maybe_flush(&mut self) {
        if self.last_flush.elapsed() >= self.flush_interval {
            self.flush_to_db();
            self.last_flush = Instant::now();
        }
    }

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

    pub fn record_click(&mut self, button_role: &str) {
        self.check_midnight();
        match button_role {
            "left" => self.stats.left_clicks += 1,
            "middle" => self.stats.middle_clicks += 1,
            "right" => self.stats.right_clicks += 1,
            "side_back" => self.stats.side_back_clicks += 1,
            "side_forward" => self.stats.side_forward_clicks += 1,
            _ => {}
        }
        self.cps_tracker.record();
        self.update_peaks();
        self.maybe_flush();
    }

    pub fn add_mouse_distance(&mut self, dx: f64, dy: f64) {
        self.check_midnight();
        self.mouse_coalesce.0 += dx;
        self.mouse_coalesce.1 += dy;
        self.stats.mouse_distance += (dx.powi(2) + dy.powi(2)).sqrt();
        self.maybe_flush();
    }

    pub fn add_scroll_distance(&mut self, delta: f64) {
        self.check_midnight();
        self.stats.scroll_distance += delta.abs();
        self.maybe_flush();
    }

    fn update_peaks(&mut self) {
        self.stats.current_kps = self.kps_tracker.current_rate();
        self.stats.current_cps = self.cps_tracker.current_rate();
        if self.stats.current_kps > self.stats.peak_kps {
            self.stats.peak_kps = self.stats.current_kps;
        }
        if self.stats.current_cps > self.stats.peak_cps {
            self.stats.peak_cps = self.stats.current_cps;
        }
        self.stats.updated_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    }

    // --- Persistence ---

    fn flush_to_db(&mut self) {
        self.stats.updated_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        if let Err(e) = db::schema::upsert_daily_stats(&self.db, &self.stats) {
            tracing::warn!("Failed to flush daily stats: {}", e);
        }
        if !self.pending_keys.is_empty() {
            match db::schema::batch_incr_key_counts(&mut self.db, &self.today, &self.pending_keys) {
                Ok(()) => { self.pending_keys.clear(); }
                Err(e) => { tracing::warn!("Failed to flush key counts: {}", e); }
            }
        }
    }

    pub fn force_flush(&mut self) {
        self.flush_to_db();
    }

    // --- Queries ---

    pub fn snapshot(&self) -> &DailyStats {
        &self.stats
    }

    pub fn rates(&self) -> RatesSnapshot {
        RatesSnapshot {
            current_kps: self.stats.current_kps,
            current_cps: self.stats.current_cps,
            peak_kps: self.stats.peak_kps,
            peak_cps: self.stats.peak_cps,
        }
    }

    pub fn history(&self, days: u32) -> Result<Vec<DailyStats>, rusqlite::Error> {
        db::schema::load_history(&self.db, days)
    }

    // --- Import / Export / Reset ---

    pub fn export_data(&self) -> Result<String, serde_json::Error> {
        let history = db::schema::load_history(&self.db, 365).unwrap_or_default();
        keystats_core::export_to_json(self.stats.clone(), history)
    }

    pub fn import_data(
        &mut self,
        json: &str,
        mode: keystats_core::ImportMode,
    ) -> Result<(), keystats_core::ImportError> {
        let imported = keystats_core::import_from_json(json)?;
        match mode {
            keystats_core::ImportMode::Overwrite => {
                self.stats = imported.today;
                self.pending_keys.clear();
                self.flush_to_db();
            }
            keystats_core::ImportMode::Merge => {
                self.stats.key_presses += imported.today.key_presses;
                self.stats.left_clicks += imported.today.left_clicks;
                self.stats.right_clicks += imported.today.right_clicks;
                self.stats.side_back_clicks += imported.today.side_back_clicks;
                self.stats.side_forward_clicks += imported.today.side_forward_clicks;
                self.stats.mouse_distance += imported.today.mouse_distance;
                self.stats.scroll_distance += imported.today.scroll_distance;
                if imported.today.peak_kps > self.stats.peak_kps {
                    self.stats.peak_kps = imported.today.peak_kps;
                }
                if imported.today.peak_cps > self.stats.peak_cps {
                    self.stats.peak_cps = imported.today.peak_cps;
                }
                self.pending_keys.clear();
                self.flush_to_db();
            }
        }
        Ok(())
    }

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

    pub fn top_keys(&self, limit: u32) -> Result<Vec<keystats_core::KeyCount>, rusqlite::Error> {
        db::schema::top_keys(&self.db, &self.today, limit)
    }

    pub fn top_keys_for_date(&self, date: &str, limit: u32) -> Result<Vec<keystats_core::KeyCount>, rusqlite::Error> {
        db::schema::top_keys(&self.db, date, limit)
    }

    pub fn clear_all_data(&mut self) {
        self.reset_today();
        if let Err(e) = db::schema::delete_all(&self.db) {
            tracing::warn!("Failed to clear all data: {}", e);
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_mgr() -> StatsManager {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::schema::migrate(&conn).unwrap();
        StatsManager::new_with_conn(conn).unwrap()
    }

    #[test]
    fn key_press_increments_count() {
        let mut mgr = test_mgr();
        mgr.record_key_press("A", true);
        mgr.record_key_press("B", true);
        assert_eq!(mgr.snapshot().key_presses, 2);
    }

    #[test]
    fn click_records_correct_button() {
        let mut mgr = test_mgr();
        mgr.record_click("left");
        mgr.record_click("left");
        mgr.record_click("right");
        assert_eq!(mgr.snapshot().left_clicks, 2);
        assert_eq!(mgr.snapshot().right_clicks, 1);
    }

    #[test]
    fn mouse_distance_accumulates() {
        let mut mgr = test_mgr();
        mgr.add_mouse_distance(3.0, 4.0);
        assert!((mgr.snapshot().mouse_distance - 5.0).abs() < 0.01);
    }

    #[test]
    fn scroll_distance_uses_absolute_value() {
        let mut mgr = test_mgr();
        mgr.add_scroll_distance(-10.0);
        mgr.add_scroll_distance(5.0);
        assert!((mgr.snapshot().scroll_distance - 15.0).abs() < 0.01);
    }

    #[test]
    fn reset_clears_today() {
        let mut mgr = test_mgr();
        mgr.record_key_press("A", true);
        mgr.record_click("left");
        mgr.reset_today();
        assert_eq!(mgr.snapshot().key_presses, 0);
        assert_eq!(mgr.snapshot().total_clicks(), 0);
    }

    #[test]
    fn import_overwrite_replaces_stats() {
        let mut mgr = test_mgr();
        let json = r#"{"version":1,"exported_at":"2026-05-26T00:00:00","today":{"date":"2026-05-26","key_presses":999,"left_clicks":100,"middle_clicks":0,"right_clicks":50,"side_back_clicks":0,"side_forward_clicks":0,"mouse_distance":0.0,"scroll_distance":0.0,"peak_kps":0,"peak_cps":0,"updated_at":"2026-05-26T00:00:00"},"history":[]}"#;
        mgr.import_data(json, keystats_core::ImportMode::Overwrite)
            .unwrap();
        assert_eq!(mgr.snapshot().key_presses, 999);
        assert_eq!(mgr.snapshot().left_clicks, 100);
    }

    #[test]
    fn import_merge_adds_stats() {
        let mut mgr = test_mgr();
        mgr.record_key_press("A", true);
        mgr.record_key_press("B", true);
        let json = r#"{"version":1,"exported_at":"2026-05-26T00:00:00","today":{"date":"2026-05-26","key_presses":10,"left_clicks":5,"middle_clicks":0,"right_clicks":3,"side_back_clicks":0,"side_forward_clicks":0,"mouse_distance":0.0,"scroll_distance":0.0,"peak_kps":0,"peak_cps":0,"updated_at":"2026-05-26T00:00:00"},"history":[]}"#;
        mgr.import_data(json, keystats_core::ImportMode::Merge)
            .unwrap();
        assert_eq!(mgr.snapshot().key_presses, 12); // 2 + 10
        assert_eq!(mgr.snapshot().left_clicks, 5);
    }

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
}
