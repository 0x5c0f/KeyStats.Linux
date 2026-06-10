use serde::{Deserialize, Serialize};

use crate::DailyStats;

/// Linux export payload version 1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPayload {
    /// Schema version (currently `1`).
    pub version: u32,
    /// ISO-8601 timestamp of when the export was created.
    pub exported_at: String,
    /// Today's statistics snapshot.
    pub today: DailyStats,
    /// Historical daily stats (newest first).
    pub history: Vec<DailyStats>,
}

/// Build an [`ExportPayload`] with the current timestamp.
pub fn create_export(today: DailyStats, history: Vec<DailyStats>) -> ExportPayload {
    ExportPayload {
        version: 1,
        exported_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        today,
        history,
    }
}

/// Serialize today's stats and history to a pretty-printed JSON string.
pub fn export_to_json(
    today: DailyStats,
    history: Vec<DailyStats>,
) -> Result<String, serde_json::Error> {
    let payload = create_export(today, history);
    serde_json::to_string_pretty(&payload)
}

/// How imported data should be merged with existing stats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// Replace all existing stats with imported data.
    Overwrite,
    /// Add imported counts to existing stats.
    Merge,
}

/// Parse a JSON string into an [`ExportPayload`], validating the version field.
pub fn import_from_json(json: &str) -> Result<ExportPayload, ImportError> {
    serde_json::from_str::<ExportPayload>(json).map_err(ImportError::Parse)
}

/// Errors that can occur when importing stats from JSON.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    /// The input string is not valid JSON or has an unexpected schema.
    #[error("JSON parse error: {0}")]
    Parse(#[from] serde_json::Error),
    /// The export version is not supported by this build.
    #[error("Unsupported export version: {0}")]
    UnsupportedVersion(u32),
}

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
