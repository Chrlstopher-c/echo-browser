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
                latest_in(db, PAGE)?
            } else {
                matching(db, terms.trim())?
            };
            Ok((entries, total))
        })
        .unwrap_or_default()
}

fn latest_in(db: &Connection, limit: usize) -> rusqlite::Result<Vec<Entry>> {
    let mut statement = db.prepare(
        "SELECT url, title, favicon, MAX(visited_at) AS seen, COUNT(*) AS visits
         FROM history GROUP BY url ORDER BY seen DESC LIMIT ?1",
    )?;
    collect(statement.query_map(params![limit as i64], row_to_entry)?)
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

/// Retire les visites d'une adresse jusqu'a `visited_at`, et s'en souvient pour les autres machines du compte.
pub fn remove(library: &Library, url: &str, visited_at: i64) -> bool {
    forget(library, url, visited_at)
}

/// Efface tout l'historique ; les autres machines du compte effaceront aussi ce qui precede.
pub fn clear(library: &Library) -> bool {
    forget(library, ALL, now())
}

/// Adresse speciale des effacements : `*` couvre toutes les adresses.
pub const ALL: &str = "*";
/// Les effacements plus vieux que cela ne sont plus transmis.
const FORGOTTEN_KEPT_S: i64 = 90 * 24 * 3600;

/// Efface les visites de `url` (ou de toutes, avec `ALL`) anterieures ou egales a `at`, et note l'effacement.
pub fn forget(library: &Library, url: &str, at: i64) -> bool {
    library
        .with(|db| {
            if url == ALL {
                db.execute("DELETE FROM history WHERE visited_at <= ?1", params![at])?;
            } else {
                db.execute("DELETE FROM history WHERE url = ?1 AND visited_at <= ?2", params![url, at])?;
            }
            db.execute(
                "INSERT INTO history_forgotten (url, at) VALUES (?1, ?2)
                 ON CONFLICT (url) DO UPDATE SET at = MAX(at, excluded.at)",
                params![url, at],
            )
        })
        .is_some()
}

/// Les effacements recents (adresse ou `ALL`, date), pour les transmettre aux autres machines.
pub fn forgotten(library: &Library) -> Vec<(String, i64)> {
    library
        .with(|db| {
            db.execute("DELETE FROM history_forgotten WHERE at < ?1", params![now() - FORGOTTEN_KEPT_S])?;
            let mut statement = db.prepare("SELECT url, at FROM history_forgotten")?;
            let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            rows.collect()
        })
        .unwrap_or_default()
}

/// Les `limit` adresses visitees le plus recemment (une entree par adresse, sa derniere visite).
pub fn latest(library: &Library, limit: usize) -> Vec<Entry> {
    library.with(|db| latest_in(db, limit)).unwrap_or_default()
}

/// Ajoute une visite venue d'une autre machine (sans doublon), sauf si elle a ete effacee ici depuis.
pub fn import(library: &Library, url: &str, title: &str, visited_at: i64) -> bool {
    library
        .with(|db| {
            db.execute(
                "INSERT OR IGNORE INTO history (url, title, favicon, visited_at)
                 SELECT ?1, ?2, NULL, ?3
                 WHERE NOT EXISTS (SELECT 1 FROM history_forgotten WHERE url IN (?1, '*') AND at >= ?3)",
                params![url, title, visited_at],
            )
        })
        .is_some()
}
