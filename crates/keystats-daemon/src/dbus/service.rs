use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use keystats_core::ImportMode;
use zbus::blocking::Connection;
use zbus::interface;

use crate::permissions;
use crate::stats::manager::StatsManager;

pub struct KeyStatsService {
    stats: Arc<Mutex<StatsManager>>,
}

impl KeyStatsService {
    pub fn new(stats: Arc<Mutex<StatsManager>>) -> Self {
        Self { stats }
    }

    /// Start the D-Bus service on a background thread. Returns the JoinHandle.
    pub fn start(stats: Arc<Mutex<StatsManager>>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let service = Self::new(stats);

            let connection = Connection::session().expect("Failed to connect to session D-Bus");

            connection
                .request_name("io.github.0x5c0f.KeyStats")
                .expect("Failed to register D-Bus service name");

            connection
                .object_server()
                .at("/io/github/0x5c0f/KeyStats", service)
                .expect("Failed to register D-Bus object");

            tracing::info!("D-Bus service registered: io.github.0x5c0f.KeyStats");

            // Keep the connection alive
            std::thread::park();
        })
    }
}

#[interface(name = "io.github.0x5c0f.KeyStats1")]
impl KeyStatsService {
    fn get_today_stats(&self) -> HashMap<String, zbus::zvariant::Value<'static>> {
        let mgr = self.stats.lock().unwrap();
        let s = mgr.snapshot();
        let mut stats = HashMap::new();
        stats.insert(
            "date".into(),
            zbus::zvariant::Value::Str(s.date.clone().into()),
        );
        stats.insert(
            "keyPresses".into(),
            zbus::zvariant::Value::U64(s.key_presses),
        );
        stats.insert(
            "leftClicks".into(),
            zbus::zvariant::Value::U64(s.left_clicks),
        );
        stats.insert(
            "middleClicks".into(),
            zbus::zvariant::Value::U64(s.middle_clicks),
        );
        stats.insert(
            "rightClicks".into(),
            zbus::zvariant::Value::U64(s.right_clicks),
        );
        stats.insert(
            "sideBackClicks".into(),
            zbus::zvariant::Value::U64(s.side_back_clicks),
        );
        stats.insert(
            "sideForwardClicks".into(),
            zbus::zvariant::Value::U64(s.side_forward_clicks),
        );
        stats.insert(
            "totalClicks".into(),
            zbus::zvariant::Value::U64(s.total_clicks()),
        );
        stats.insert(
            "mouseDistance".into(),
            zbus::zvariant::Value::F64(s.mouse_distance),
        );
        stats.insert(
            "scrollDistance".into(),
            zbus::zvariant::Value::F64(s.scroll_distance),
        );
        stats.insert(
            "currentKPS".into(),
            zbus::zvariant::Value::U32(s.current_kps),
        );
        stats.insert(
            "currentCPS".into(),
            zbus::zvariant::Value::U32(s.current_cps),
        );
        stats.insert("peakKPS".into(), zbus::zvariant::Value::U32(s.peak_kps));
        stats.insert("peakCPS".into(), zbus::zvariant::Value::U32(s.peak_cps));
        stats.insert(
            "updatedAt".into(),
            zbus::zvariant::Value::Str(s.updated_at.clone().into()),
        );
        stats
    }

    fn get_rates(&self) -> HashMap<String, zbus::zvariant::Value<'static>> {
        let mgr = self.stats.lock().unwrap();
        let r = mgr.rates();
        let mut rates = HashMap::new();
        rates.insert(
            "currentKPS".into(),
            zbus::zvariant::Value::U32(r.current_kps),
        );
        rates.insert(
            "currentCPS".into(),
            zbus::zvariant::Value::U32(r.current_cps),
        );
        rates.insert("peakKPS".into(), zbus::zvariant::Value::U32(r.peak_kps));
        rates.insert("peakCPS".into(), zbus::zvariant::Value::U32(r.peak_cps));
        rates
    }

    fn get_history(&self, days: u32) -> String {
        let mgr = self.stats.lock().unwrap();
        match mgr.history(days) {
            Ok(history) => serde_json::to_string(&history).unwrap_or_default(),
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    fn get_permission_status(&self) -> String {
        let status = permissions::diagnose();
        serde_json::to_string(&status).unwrap_or_default()
    }

    fn get_top_keys(&self, limit: u32) -> String {
        let mgr = self.stats.lock().unwrap();
        match mgr.top_keys(limit) {
            Ok(keys) => serde_json::to_string(&keys).unwrap_or_default(),
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    fn reset_today(&self) -> bool {
        let mut mgr = self.stats.lock().unwrap();
        mgr.reset_today();
        true
    }

    fn clear_all_data(&self) -> bool {
        let mut mgr = self.stats.lock().unwrap();
        mgr.clear_all_data();
        true
    }

    fn export_data(&self) -> String {
        let mgr = self.stats.lock().unwrap();
        mgr.export_data().unwrap_or_default()
    }

    fn import_data(&self, json: &str, mode: &str) -> bool {
        let import_mode = match mode {
            "overwrite" => ImportMode::Overwrite,
            _ => ImportMode::Merge,
        };
        let mut mgr = self.stats.lock().unwrap();
        mgr.import_data(json, import_mode).is_ok()
    }

    #[zbus(property)]
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}
