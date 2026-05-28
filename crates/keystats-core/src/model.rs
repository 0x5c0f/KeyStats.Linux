use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyStats {
    pub date: String,
    pub key_presses: u64,
    pub left_clicks: u64,
    pub middle_clicks: u64,
    pub right_clicks: u64,
    pub side_back_clicks: u64,
    pub side_forward_clicks: u64,
    pub mouse_distance: f64,
    pub scroll_distance: f64,
    #[serde(default)]
    pub current_kps: u32,
    #[serde(default)]
    pub current_cps: u32,
    pub peak_kps: u32,
    pub peak_cps: u32,
    pub updated_at: String,
}

impl DailyStats {
    pub fn today() -> Self {
        Self {
            date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            updated_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            ..Default::default()
        }
    }

    pub fn total_clicks(&self) -> u64 {
        self.left_clicks
            + self.middle_clicks
            + self.right_clicks
            + self.side_back_clicks
            + self.side_forward_clicks
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RatesSnapshot {
    pub current_kps: u32,
    pub current_cps: u32,
    pub peak_kps: u32,
    pub peak_cps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionStatus {
    pub can_read_any_input: bool,
    pub readable_devices: u32,
    pub blocked_devices: u32,
    pub recommended_action: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub show_keys: bool,
    pub show_clicks: bool,
    pub refresh_interval_ms: u64,
    pub dynamic_color: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_keys: true,
            show_clicks: true,
            refresh_interval_ms: 1000,
            dynamic_color: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyCount {
    pub key_name: String,
    pub count: u64,
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
