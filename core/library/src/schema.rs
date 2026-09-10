//! Responsabilite : la forme de la base et sa mise a niveau.

use rusqlite::Connection;
use tracing::info;

/// Version du schema. A incrementer en ajoutant la migration correspondante.
const VERSION: i32 = 1;

/// Prepare la base : cree ce qui manque, met a niveau ce qui est ancien.
pub fn prepare(connection: &Connection) -> rusqlite::Result<()> {
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "foreign_keys", "ON")?;

    let current: i32 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current >= VERSION {
        return Ok(());
    }
    info!(depuis = current, vers = VERSION, "mise a niveau de la bibliotheque");

    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS bookmarks (
            url        TEXT PRIMARY KEY,
            title      TEXT NOT NULL,
            favicon    TEXT,
            added_at   INTEGER NOT NULL,
            position   INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS history (
            url        TEXT NOT NULL,
            title      TEXT NOT NULL,
            favicon    TEXT,
            visited_at INTEGER NOT NULL,
            PRIMARY KEY (url, visited_at)
         );
         CREATE INDEX IF NOT EXISTS history_by_date ON history (visited_at DESC);
         CREATE TABLE IF NOT EXISTS downloads (
            id         INTEGER PRIMARY KEY,
            file_name  TEXT NOT NULL,
            url        TEXT NOT NULL,
            path       TEXT,
            received   INTEGER NOT NULL DEFAULT 0,
            total      INTEGER,
            state      TEXT NOT NULL,
            started_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
         );",
    )?;
    connection.pragma_update(None, "user_version", VERSION)?;
    Ok(())
}
