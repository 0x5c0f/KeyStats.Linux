use serde::{Deserialize, Serialize};

use crate::DailyStats;

/// Linux export payload version 1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPayload {
    pub version: u32,
    pub exported_at: String,
    pub today: DailyStats,
    pub history: Vec<DailyStats>,
}

pub fn create_export(today: DailyStats, history: Vec<DailyStats>) -> ExportPayload {
    ExportPayload {
        version: 1,
        exported_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        today,
        history,
    }
}

pub fn export_to_json(
    today: DailyStats,
    history: Vec<DailyStats>,
) -> Result<String, serde_json::Error> {
    let payload = create_export(today, history);
    serde_json::to_string_pretty(&payload)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    Overwrite,
    Merge,
}

pub fn import_from_json(json: &str) -> Result<ExportPayload, ImportError> {
    serde_json::from_str::<ExportPayload>(json).map_err(ImportError::Parse)
}

#[derive(Debug)]
pub enum ImportError {
    Parse(serde_json::Error),
    UnsupportedVersion(u32),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::Parse(e) => write!(f, "JSON parse error: {e}"),
            ImportError::UnsupportedVersion(v) => {
                write!(f, "Unsupported export version: {v}")
            }
        }
    }
}

impl std::error::Error for ImportError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_stats() -> DailyStats {
        DailyStats {
            key_presses: 100,
            left_clicks: 50,
            right_clicks: 10,
            mouse_distance: 128.5,
            ..DailyStats::today()
        }
    }

    #[test]
    fn roundtrip_export_import() {
        let today = sample_stats();
        let history = vec![DailyStats::default()];
        let json = export_to_json(today.clone(), history.clone()).unwrap();
        let imported = import_from_json(&json).unwrap();
        assert_eq!(imported.version, 1);
        assert_eq!(imported.today.key_presses, today.key_presses);
        assert_eq!(imported.today.left_clicks, today.left_clicks);
        assert_eq!(imported.history.len(), 1);
    }

    #[test]
    fn import_malformed_json_fails() {
        assert!(import_from_json("not json").is_err());
    }

    #[test]
    fn import_empty_json_fails() {
        assert!(import_from_json("").is_err());
    }

    #[test]
    fn export_contains_timestamp() {
        let json = export_to_json(sample_stats(), vec![]).unwrap();
        assert!(json.contains("exported_at"));
        assert!(json.contains("\"version\": 1"));
    }
}
