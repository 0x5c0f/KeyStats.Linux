use serde::{Deserialize, Serialize};

/// Daily aggregated keyboard and mouse statistics for a single date.
///
/// Stores per-day counters for key presses, mouse clicks (by button),
/// mouse travel distance, scroll distance, and instantaneous rate peaks.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyStats {
    /// Date string in `YYYY-MM-DD` format (local timezone).
    pub date: String,
    /// Total key presses recorded today.
    pub key_presses: u64,
    /// Left mouse button click count.
    pub left_clicks: u64,
    /// Middle mouse button click count.
    pub middle_clicks: u64,
    /// Right mouse button click count.
    pub right_clicks: u64,
    /// Side/back mouse button click count.
    pub side_back_clicks: u64,
    /// Side/forward mouse button click count.
    pub side_forward_clicks: u64,
    /// Cumulative mouse pointer travel distance in device units.
    pub mouse_distance: f64,
    /// Cumulative scroll wheel distance in device units.
    pub scroll_distance: f64,
    /// Current keys-per-second (instantaneous).
    #[serde(default)]
    pub current_kps: u32,
    /// Current clicks-per-second (instantaneous).
    #[serde(default)]
    pub current_cps: u32,
    /// Peak keys-per-second observed today.
    pub peak_kps: u32,
    /// Peak clicks-per-second observed today.
    pub peak_cps: u32,
    /// ISO-8601 timestamp of the last update.
    pub updated_at: String,
}

impl DailyStats {
    /// Create a new `DailyStats` initialized for today's date with zero counters.
    pub fn today() -> Self {
        Self {
            date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            updated_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            ..Default::default()
        }
    }

    /// Sum of all mouse button click counts.
    pub fn total_clicks(&self) -> u64 {
        self.left_clicks
            + self.middle_clicks
            + self.right_clicks
            + self.side_back_clicks
            + self.side_forward_clicks
    }
}

/// Point-in-time snapshot of keystroke and click rates.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RatesSnapshot {
    /// Current keys-per-second.
    pub current_kps: u32,
    /// Current clicks-per-second.
    pub current_cps: u32,
    /// Peak keys-per-second today.
    pub peak_kps: u32,
    /// Peak clicks-per-second today.
    pub peak_cps: u32,
}

/// Diagnostic result for input device permissions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionStatus {
    /// Whether at least one input device is readable.
    pub can_read_any_input: bool,
    /// Number of successfully opened input devices.
    pub readable_devices: u32,
    /// Number of devices that returned `PermissionDenied`.
    pub blocked_devices: u32,
    /// Suggested fix (e.g. `sudo usermod -aG input $USER`).
    pub recommended_action: String,
    /// Human-readable status summary.
    pub message: String,
}

/// User-configurable display and behavior settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Whether to show key-press statistics in the panel.
    pub show_keys: bool,
    /// Whether to show mouse-click statistics in the panel.
    pub show_clicks: bool,
    /// Panel refresh interval in milliseconds.
    pub refresh_interval_ms: u64,
    /// Whether the panel icon color changes with activity.
    pub dynamic_color: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { show_keys: true, show_clicks: true, refresh_interval_ms: 1000, dynamic_color: false }
    }
}

/// A single key's press count for a given date.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyCount {
    /// Display name of the key (e.g. `"A"`, `"Space"`, `"LeftShift"`).
    pub key_name: String,
    /// Number of times the key was pressed.
    pub count: u64,
}

/// Real-time input event for D-Bus signal transmission.
///
/// Emitted by the daemon's event loop and consumed by the overlay process
/// to display keystrokes in real-time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    /// A key was pressed. The name may include modifiers (e.g. `"Ctrl+C"`).
    KeyPress {
        /// Human-readable key name.
        name: String,
    },
    /// A key was released.
    KeyRelease {
        /// Human-readable key name matching the corresponding press event.
        name: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_stats_defaults() {
        let stats = DailyStats::default();
        assert_eq!(stats.key_presses, 0);
        assert_eq!(stats.total_clicks(), 0);
    }

    #[test]
    fn total_clicks_sums_all_buttons() {
        let stats = DailyStats {
            left_clicks: 10,
            middle_clicks: 3,
            right_clicks: 5,
            side_back_clicks: 2,
            side_forward_clicks: 1,
            ..Default::default()
        };
        assert_eq!(stats.total_clicks(), 21);
    }

    #[test]
    fn today_uses_current_date() {
        let stats = DailyStats::today();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(stats.date, today);
        assert!(!stats.updated_at.is_empty());
    }

    #[test]
    fn settings_default() {
        let s = Settings::default();
        assert!(s.show_keys);
        assert!(s.show_clicks);
        assert_eq!(s.refresh_interval_ms, 1000);
    }
}
