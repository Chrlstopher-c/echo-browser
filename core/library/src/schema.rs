//! Responsabilite : la forme de la base et sa mise a niveau.

use rusqlite::Connection;
use tracing::info;

/// Version du schema. A incrementer en ajoutant la migration correspondante.
const VERSION: i32 = 3;

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
         CREATE TABLE IF NOT EXISTS watched_pages (
            url     TEXT PRIMARY KEY,
            text    TEXT,
            checked INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS signals_day (
            day  TEXT NOT NULL,
            kind TEXT NOT NULL,
            key  TEXT NOT NULL,
            n    INTEGER NOT NULL,
            PRIMARY KEY (day, kind, key)
         );
         CREATE TABLE IF NOT EXISTS hidden_elements (
            fingerprint TEXT NOT NULL,
            selector    TEXT NOT NULL,
            site        TEXT NOT NULL,
            created     INTEGER NOT NULL,
            PRIMARY KEY (fingerprint, selector)
         );
         CREATE TABLE IF NOT EXISTS sequences (
            fingerprint TEXT PRIMARY KEY,
            urls        TEXT NOT NULL,
            n           INTEGER NOT NULL,
            last        INTEGER NOT NULL,
            state       TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS routines (
            id      INTEGER PRIMARY KEY,
            name    TEXT NOT NULL,
            urls    TEXT NOT NULL,
            created INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS site_journal (
            site   TEXT NOT NULL,
            at     INTEGER NOT NULL,
            kind   TEXT NOT NULL,
            detail TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS site_journal_by_site ON site_journal (site, at DESC);
         CREATE TABLE IF NOT EXISTS history_forgotten (
            url TEXT PRIMARY KEY,
            at  INTEGER NOT NULL
         );
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
         );
         CREATE TABLE IF NOT EXISTS permissions (
            origin TEXT NOT NULL,
            kind   TEXT NOT NULL,
            allow  INTEGER NOT NULL,
            PRIMARY KEY (origin, kind)
         );",
    )?;
    // Version 3 : historique et favoris propres a chaque profil. Les donnees d'avant vont au profil principal.
    for table in ["history", "bookmarks"] {
        add_column(connection, table, "space", "TEXT NOT NULL DEFAULT 'graphite'")?;
    }
    connection.pragma_update(None, "user_version", VERSION)?;
    Ok(())
}

/// Ajoute une colonne si elle manque (SQLite n'a pas d'`ADD COLUMN IF NOT EXISTS`).
fn add_column(connection: &Connection, table: &str, column: &str, definition: &str) -> rusqlite::Result<()> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let exists = statement.query_map([], |row| row.get::<_, String>(1))?.flatten().any(|name| name == column);
    if !exists {
        connection.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"))?;
    }
    Ok(())
}
