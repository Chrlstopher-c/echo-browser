//! Responsabilite : l'historique de navigation, sa recherche et son effacement.

use crate::{now, Library};
use rusqlite::{params, Connection};

#[derive(Debug, Clone)]
pub struct Entry {
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub visited_at: i64,
    /// Nombre total de visites sur cette adresse.
    pub visits: u32,
}

/// Nombre d'entrees rendues par defaut.
const PAGE: usize = 200;

/// Enregistre une visite. Les pages internes n'y figurent pas.
pub fn record(library: &Library, url: &str, title: &str, favicon: Option<&str>) -> bool {
    if url.is_empty() || url.starts_with("echo://") || url.starts_with("chrome://") {
        return false;
    }
    library
        .with(|db| {
            db.execute(
                "INSERT OR REPLACE INTO history (url, title, favicon, visited_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![url, title, favicon, now()],
            )
        })
        .is_some()
}

/// Les entrees les plus recentes, filtrees par `terms` si la chaine n'est pas vide.
/// Rend aussi le nombre total d'adresses distinctes.
pub fn search(library: &Library, terms: &str) -> (Vec<Entry>, usize) {
    library
        .with(|db| {
            let total: usize =
                db.query_row("SELECT COUNT(DISTINCT url) FROM history", [], |row| row.get(0))?;
            let entries = if terms.trim().is_empty() {
                recent(db)?
            } else {
                matching(db, terms.trim())?
            };
            Ok((entries, total))
        })
        .unwrap_or_default()
}

fn recent(db: &Connection) -> rusqlite::Result<Vec<Entry>> {
    let mut statement = db.prepare(
        "SELECT url, title, favicon, MAX(visited_at) AS seen, COUNT(*) AS visits
         FROM history GROUP BY url ORDER BY seen DESC LIMIT ?1",
    )?;
    collect(statement.query_map(params![PAGE as i64], row_to_entry)?)
}

fn matching(db: &Connection, terms: &str) -> rusqlite::Result<Vec<Entry>> {
    let pattern = format!("%{}%", terms.replace('%', "\\%"));
    let mut statement = db.prepare(
        "SELECT url, title, favicon, MAX(visited_at) AS seen, COUNT(*) AS visits
         FROM history WHERE url LIKE ?1 ESCAPE '\\' OR title LIKE ?1 ESCAPE '\\'
         GROUP BY url ORDER BY seen DESC LIMIT ?2",
    )?;
    collect(statement.query_map(params![pattern, PAGE as i64], row_to_entry)?)
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    Ok(Entry {
        url: row.get(0)?,
        title: row.get(1)?,
        favicon: row.get(2)?,
        visited_at: row.get(3)?,
        visits: row.get::<_, i64>(4)? as u32,
    })
}

fn collect<'a>(
    rows: impl Iterator<Item = rusqlite::Result<Entry>> + 'a,
) -> rusqlite::Result<Vec<Entry>> {
    rows.collect()
}

/// Retire une visite precise.
pub fn remove(library: &Library, url: &str, visited_at: i64) -> bool {
    library
        .with(|db| {
            db.execute(
                "DELETE FROM history WHERE url = ?1 AND visited_at <= ?2",
                params![url, visited_at],
            )
        })
        .is_some()
}

pub fn clear(library: &Library) -> bool {
    library.with(|db| db.execute("DELETE FROM history", [])).is_some()
}
