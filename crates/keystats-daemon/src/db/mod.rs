//! SQLite database access layer.

pub mod schema;

use rusqlite::Connection;
use std::path::PathBuf;

/// Return the default database path under `$XDG_STATE_HOME/keystats/stats.sqlite3`.
fn db_path() -> Result<PathBuf, std::env::VarError> {
    let base = match std::env::var_os("XDG_STATE_HOME") {
        Some(val) => PathBuf::from(val),
        None => PathBuf::from(std::env::var("HOME")?).join(".local").join("state"),
    };
    Ok(base.join("keystats").join("stats.sqlite3"))
}

/// Open (or create) the SQLite database, enable WAL mode, and run migrations.
pub fn open() -> Result<Connection, rusqlite::Error> {
    let path = db_path().map_err(|e| {
        rusqlite::Error::InvalidParameterName(format!("Failed to determine DB path: {e}"))
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(&path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    schema::migrate(&conn)?;
    Ok(conn)
}
