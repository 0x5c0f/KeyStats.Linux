use rusqlite::Connection;
use std::collections::HashMap;

const SCHEMA_VERSION: i32 = 2;

/// Run incremental schema migrations based on `user_version` pragma.
pub fn migrate(conn: &Connection) -> Result<(), rusqlite::Error> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0)).unwrap_or(0);

    if version < 1 {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS daily_stats (
                date TEXT PRIMARY KEY,
                key_presses INTEGER NOT NULL DEFAULT 0,
                left_clicks INTEGER NOT NULL DEFAULT 0,
                middle_clicks INTEGER NOT NULL DEFAULT 0,
                right_clicks INTEGER NOT NULL DEFAULT 0,
                side_back_clicks INTEGER NOT NULL DEFAULT 0,
                side_forward_clicks INTEGER NOT NULL DEFAULT 0,
                mouse_distance REAL NOT NULL DEFAULT 0,
                scroll_distance REAL NOT NULL DEFAULT 0,
                peak_kps INTEGER NOT NULL DEFAULT 0,
                peak_cps INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS key_counts (
                date TEXT NOT NULL,
                key_name TEXT NOT NULL,
                count INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (date, key_name)
            );

            CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    }

    if version < 2 {
        conn.execute_batch(
            "ALTER TABLE daily_stats ADD COLUMN middle_clicks INTEGER NOT NULL DEFAULT 0;",
        )
        .ok();
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    }

    Ok(())
}

/// Insert or update daily stats for a given date.
pub fn upsert_daily_stats(
    conn: &Connection,
    stats: &keystats_core::DailyStats,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO daily_stats (date, key_presses, left_clicks, middle_clicks, right_clicks,
         side_back_clicks, side_forward_clicks, mouse_distance, scroll_distance,
         peak_kps, peak_cps, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, datetime('now'))
         ON CONFLICT(date) DO UPDATE SET
         key_presses = excluded.key_presses,
         left_clicks = excluded.left_clicks,
         middle_clicks = excluded.middle_clicks,
         right_clicks = excluded.right_clicks,
         side_back_clicks = excluded.side_back_clicks,
         side_forward_clicks = excluded.side_forward_clicks,
         mouse_distance = excluded.mouse_distance,
         scroll_distance = excluded.scroll_distance,
         peak_kps = MAX(peak_kps, excluded.peak_kps),
         peak_cps = MAX(peak_cps, excluded.peak_cps),
         updated_at = datetime('now')",
        rusqlite::params![
            stats.date,
            stats.key_presses,
            stats.left_clicks,
            stats.middle_clicks,
            stats.right_clicks,
            stats.side_back_clicks,
            stats.side_forward_clicks,
            stats.mouse_distance,
            stats.scroll_distance,
            stats.peak_kps,
            stats.peak_cps,
        ],
    )?;
    Ok(())
}

/// Load daily stats for a specific date, returning `None` if no record exists.
pub fn load_daily_stats(
    conn: &Connection,
    date: &str,
) -> Result<Option<keystats_core::DailyStats>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT date, key_presses, left_clicks, middle_clicks, right_clicks,
         side_back_clicks, side_forward_clicks, mouse_distance, scroll_distance,
         peak_kps, peak_cps, updated_at
         FROM daily_stats WHERE date = ?1",
    )?;
    let result = stmt.query_row(rusqlite::params![date], |row| {
        Ok(keystats_core::DailyStats {
            date: row.get(0)?,
            key_presses: row.get(1)?,
            left_clicks: row.get(2)?,
            middle_clicks: row.get(3)?,
            right_clicks: row.get(4)?,
            side_back_clicks: row.get(5)?,
            side_forward_clicks: row.get(6)?,
            mouse_distance: row.get(7)?,
            scroll_distance: row.get(8)?,
            current_kps: 0,
            current_cps: 0,
            peak_kps: row.get(9)?,
            peak_cps: row.get(10)?,
            updated_at: row.get(11)?,
        })
    });
    match result {
        Ok(stats) => Ok(Some(stats)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Load the last `days` days of stats, filling gaps with zeros (newest first).
pub fn load_history(
    conn: &Connection,
    days: u32,
) -> Result<Vec<keystats_core::DailyStats>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "WITH RECURSIVE dates(d) AS (
            SELECT date('now')
            UNION ALL
            SELECT date(d, '-1 day') FROM dates LIMIT ?1
        )
        SELECT dates.d,
               COALESCE(ds.key_presses, 0),
               COALESCE(ds.left_clicks, 0),
               COALESCE(ds.middle_clicks, 0),
               COALESCE(ds.right_clicks, 0),
               COALESCE(ds.side_back_clicks, 0),
               COALESCE(ds.side_forward_clicks, 0),
               COALESCE(ds.mouse_distance, 0.0),
               COALESCE(ds.scroll_distance, 0.0),
               COALESCE(ds.peak_kps, 0),
               COALESCE(ds.peak_cps, 0),
               COALESCE(ds.updated_at, '')
        FROM dates
        LEFT JOIN daily_stats ds ON dates.d = ds.date
        ORDER BY dates.d DESC",
    )?;
    let rows = stmt.query_map(rusqlite::params![days], |row| {
        Ok(keystats_core::DailyStats {
            date: row.get(0)?,
            key_presses: row.get(1)?,
            left_clicks: row.get(2)?,
            middle_clicks: row.get(3)?,
            right_clicks: row.get(4)?,
            side_back_clicks: row.get(5)?,
            side_forward_clicks: row.get(6)?,
            mouse_distance: row.get(7)?,
            scroll_distance: row.get(8)?,
            current_kps: 0,
            current_cps: 0,
            peak_kps: row.get(9)?,
            peak_cps: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?;
    rows.collect()
}

/// Atomically increment per-key counts for a given date.
pub fn batch_incr_key_counts(
    conn: &mut Connection,
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

/// Get the most-pressed keys for a date, ordered by count descending.
pub fn top_keys(
    conn: &Connection,
    date: &str,
    limit: u32,
) -> Result<Vec<keystats_core::KeyCount>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT key_name, count FROM key_counts
         WHERE date = ?1 ORDER BY count DESC LIMIT ?2",
    )?;
    let rows = stmt.query_map(rusqlite::params![date, limit], |row| {
        Ok(keystats_core::KeyCount { key_name: row.get(0)?, count: row.get(1)? })
    })?;
    rows.collect()
}

/// Delete all key breakdown records for a specific date.
pub fn delete_today_key_counts(conn: &Connection, date: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM key_counts WHERE date = ?1", rusqlite::params![date])?;
    Ok(())
}

/// Delete all data (daily stats and key counts). Use with caution.
pub fn delete_all(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch("DELETE FROM daily_stats; DELETE FROM key_counts;")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        // Verify tables exist
        let c: i64 = conn.query_row("SELECT COUNT(*) FROM daily_stats", [], |r| r.get(0)).unwrap();
        assert_eq!(c, 0);
        conn.query_row("SELECT COUNT(*) FROM key_counts", [], |r| r.get::<_, i64>(0)).unwrap();
        conn.query_row("SELECT COUNT(*) FROM metadata", [], |r| r.get::<_, i64>(0)).unwrap();
    }

    #[test]
    fn upsert_and_load() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        let s = keystats_core::DailyStats {
            date: "2026-05-26".into(),
            key_presses: 100,
            left_clicks: 10,
            right_clicks: 5,
            side_back_clicks: 2,
            side_forward_clicks: 1,
            mouse_distance: 128.5,
            scroll_distance: 42.0,
            peak_kps: 10,
            peak_cps: 5,
            ..Default::default()
        };
        upsert_daily_stats(&conn, &s).unwrap();

        let loaded = load_daily_stats(&conn, "2026-05-26").unwrap().unwrap();
        assert_eq!(loaded.key_presses, 100);
        assert_eq!(loaded.left_clicks, 10);
        assert_eq!(loaded.right_clicks, 5);
        assert_eq!(loaded.mouse_distance, 128.5);
        assert_eq!(loaded.peak_kps, 10);
    }

    #[test]
    fn upsert_updates_existing() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        let s1 = keystats_core::DailyStats {
            date: "2026-05-26".into(),
            key_presses: 100,
            left_clicks: 10,
            right_clicks: 5,
            mouse_distance: 50.0,
            scroll_distance: 10.0,
            peak_kps: 5,
            peak_cps: 3,
            ..Default::default()
        };
        upsert_daily_stats(&conn, &s1).unwrap();
        let s2 = keystats_core::DailyStats {
            date: "2026-05-26".into(),
            key_presses: 200,
            left_clicks: 20,
            right_clicks: 10,
            mouse_distance: 100.0,
            scroll_distance: 20.0,
            peak_kps: 8,
            peak_cps: 4,
            ..Default::default()
        };
        upsert_daily_stats(&conn, &s2).unwrap();

        let loaded = load_daily_stats(&conn, "2026-05-26").unwrap().unwrap();
        assert_eq!(loaded.key_presses, 200);
        // peak_kps should be MAX of both values
        assert_eq!(loaded.peak_kps, 8);
    }

    #[test]
    fn load_nonexistent_returns_none() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let result = load_daily_stats(&conn, "2026-01-01").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn history_respects_limit() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        let history = load_history(&conn, 3).unwrap();
        assert_eq!(history.len(), 3);
        // Today first, descending
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(history[0].date, today);
        // All dates are consecutive
        assert!(history[0].date > history[1].date);
        assert!(history[1].date > history[2].date);
    }

    #[test]
    fn history_fills_gaps_with_zeros() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let s = keystats_core::DailyStats {
            date: today.clone(),
            key_presses: 100,
            left_clicks: 10,
            ..Default::default()
        };
        upsert_daily_stats(&conn, &s).unwrap();

        let history = load_history(&conn, 3).unwrap();
        assert_eq!(history.len(), 3);
        // Today has data
        assert_eq!(history[0].date, today);
        assert_eq!(history[0].key_presses, 100);
        // Yesterday and day before are zeros
        assert_eq!(history[1].key_presses, 0);
        assert_eq!(history[2].key_presses, 0);
        assert_eq!(history[1].left_clicks, 0);
    }
}
