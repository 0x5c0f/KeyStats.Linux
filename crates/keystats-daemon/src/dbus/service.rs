use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use keystats_core::{ImportMode, InputEvent};
use zbus::blocking::connection;
use zbus::interface;

use crate::permissions;
use crate::stats::manager::{StatsManager, lock_stats};

/// D-Bus object path for the KeyStats service.
const OBJECT_PATH: &str = "/io/github/0x5c0f/KeyStats";
/// D-Bus well-known bus name.
const BUS_NAME: &str = "io.github.x0x5c0f.KeyStats";
/// D-Bus interface name.
const INTERFACE_NAME: &str = "io.github.x0x5c0f.KeyStats1";

pub struct KeyStatsService {
    stats: Arc<Mutex<StatsManager>>,
}

impl KeyStatsService {
    pub fn new(stats: Arc<Mutex<StatsManager>>) -> Self {
        Self { stats }
    }

    /// Start the D-Bus service on a background thread.
    ///
    /// `event_rx` receives real-time input events from the event loop.
    /// Each event is forwarded as a D-Bus signal for the overlay process.
    pub fn start(stats: Arc<Mutex<StatsManager>>, event_rx: mpsc::Receiver<InputEvent>) {
        thread::spawn(move || {
            let service = Self::new(stats);

            let connection = match connection::Builder::session()
                .and_then(|b| b.name(BUS_NAME))
                .and_then(|b| b.serve_at(OBJECT_PATH, service))
                .and_then(|b| b.build())
            {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Failed to start D-Bus service: {e}");
                    return;
                }
            };

            tracing::info!("D-Bus service registered: {BUS_NAME}");

            // Clone connection for the signal forwarding thread
            let conn_clone = connection.clone();
            thread::spawn(move || {
                forward_key_events(conn_clone, event_rx);
            });

            // Keep the connection alive
            std::thread::park();
        });
    }
}

/// Forward input events from the channel as D-Bus signals.
fn forward_key_events(connection: zbus::blocking::Connection, event_rx: mpsc::Receiver<InputEvent>) {
    while let Ok(event) = event_rx.recv() {
        match event {
            InputEvent::KeyPress { name } => {
                if let Err(e) = connection.emit_signal(
                    None::<&str>,
                    OBJECT_PATH,
                    INTERFACE_NAME,
                    "KeyPressed",
                    &(&name,),
                ) {
                    tracing::trace!("Failed to emit KeyPressed signal: {e}");
                }
            }
            InputEvent::KeyRelease { name } => {
                if let Err(e) = connection.emit_signal(
                    None::<&str>,
                    OBJECT_PATH,
                    INTERFACE_NAME,
                    "KeyReleased",
                    &(&name,),
                ) {
                    tracing::trace!("Failed to emit KeyReleased signal: {e}");
                }
            }
        }
    }
}

#[interface(name = "io.github.x0x5c0f.KeyStats1")]
impl KeyStatsService {
    fn get_today_stats(&self) -> HashMap<String, zbus::zvariant::Value<'static>> {
        let Some(mgr) = lock_stats(&self.stats) else {
            return HashMap::new();
        };
        let s = mgr.snapshot();
        let mut stats = HashMap::new();
        stats.insert("date".into(), zbus::zvariant::Value::Str(s.date.clone().into()));
        stats.insert("keyPresses".into(), zbus::zvariant::Value::U64(s.key_presses));
        stats.insert("leftClicks".into(), zbus::zvariant::Value::U64(s.left_clicks));
        stats.insert("middleClicks".into(), zbus::zvariant::Value::U64(s.middle_clicks));
        stats.insert("rightClicks".into(), zbus::zvariant::Value::U64(s.right_clicks));
        stats.insert("sideBackClicks".into(), zbus::zvariant::Value::U64(s.side_back_clicks));
        stats.insert("sideForwardClicks".into(), zbus::zvariant::Value::U64(s.side_forward_clicks));
        stats.insert("totalClicks".into(), zbus::zvariant::Value::U64(s.total_clicks()));
        stats.insert("mouseDistance".into(), zbus::zvariant::Value::F64(s.mouse_distance));
        stats.insert("scrollDistance".into(), zbus::zvariant::Value::F64(s.scroll_distance));
        stats.insert("currentKPS".into(), zbus::zvariant::Value::U32(s.current_kps));
        stats.insert("currentCPS".into(), zbus::zvariant::Value::U32(s.current_cps));
        stats.insert("peakKPS".into(), zbus::zvariant::Value::U32(s.peak_kps));
        stats.insert("peakCPS".into(), zbus::zvariant::Value::U32(s.peak_cps));
        stats.insert("updatedAt".into(), zbus::zvariant::Value::Str(s.updated_at.clone().into()));
        stats
    }

    fn get_rates(&self) -> HashMap<String, zbus::zvariant::Value<'static>> {
        let Some(mgr) = lock_stats(&self.stats) else {
            return HashMap::new();
        };
        let r = mgr.rates();
        let mut rates = HashMap::new();
        rates.insert("currentKPS".into(), zbus::zvariant::Value::U32(r.current_kps));
        rates.insert("currentCPS".into(), zbus::zvariant::Value::U32(r.current_cps));
        rates.insert("peakKPS".into(), zbus::zvariant::Value::U32(r.peak_kps));
        rates.insert("peakCPS".into(), zbus::zvariant::Value::U32(r.peak_cps));
        rates
    }

    fn get_history(&self, days: u32) -> String {
        let Some(mgr) = lock_stats(&self.stats) else {
            return String::new();
        };
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
        let Some(mgr) = lock_stats(&self.stats) else {
            return String::new();
        };
        match mgr.top_keys(limit) {
            Ok(keys) => serde_json::to_string(&keys).unwrap_or_default(),
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    fn get_top_keys_for_date(&self, date: &str, limit: u32) -> String {
        let Some(mgr) = lock_stats(&self.stats) else {
            return String::new();
        };
        match mgr.top_keys_for_date(date, limit) {
            Ok(keys) => serde_json::to_string(&keys).unwrap_or_default(),
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    fn reset_today(&self) -> bool {
        let Some(mut mgr) = lock_stats(&self.stats) else {
            return false;
        };
        mgr.reset_today();
        true
    }

    fn clear_all_data(&self) -> bool {
        let Some(mut mgr) = lock_stats(&self.stats) else {
            return false;
        };
        mgr.clear_all_data();
        true
    }

    fn export_data(&self) -> String {
        let Some(mgr) = lock_stats(&self.stats) else {
            return String::new();
        };
        mgr.export_data().unwrap_or_default()
    }

    fn import_data(&self, json: &str, mode: &str) -> bool {
        let import_mode = match mode {
            "overwrite" => ImportMode::Overwrite,
            _ => ImportMode::Merge,
        };
        let Some(mut mgr) = lock_stats(&self.stats) else {
            return false;
        };
        mgr.import_data(json, import_mode).is_ok()
    }

    #[zbus(property)]
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}
