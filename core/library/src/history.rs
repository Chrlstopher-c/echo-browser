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
    /// Profil ou la visite a eu lieu.
    pub space: String,
}

/// Nombre d'entrees rendues par defaut.
const PAGE: usize = 200;

/// Ajoute des visites venues d'un autre navigateur (adresse, titre, date en secondes). Les doublons sont ignores.
/// Rend le nombre de visites ajoutees.
pub fn import_many(library: &Library, visits: &[(String, String, i64)], space: &str) -> usize {
    library
        .with(|db| {
            let tx = db.unchecked_transaction()?;
            let mut added = 0;
            {
                let mut insert = tx.prepare(
                    "INSERT OR IGNORE INTO history (url, title, favicon, visited_at, space) VALUES (?1, ?2, NULL, ?3, ?4)",
                )?;
                for (url, title, at) in visits {
                    if url.starts_with("http") {
                        added += insert.execute(params![url, title, at, space])?;
                    }
                }
            }
            tx.commit()?;
            Ok(added)
        })
        .unwrap_or(0)
}

/// Enregistre une visite dans le profil `space`. Les pages internes n'y figurent pas.
pub fn record(library: &Library, url: &str, title: &str, favicon: Option<&str>, space: &str) -> bool {
    if url.is_empty() || url.starts_with("echo://") || url.starts_with("chrome://") {
        return false;
    }
    library
        .with(|db| {
            db.execute(
                "INSERT OR REPLACE INTO history (url, title, favicon, visited_at, space)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![url, title, favicon, now(), space],
            )
        })
        .is_some()
}

/// Les entrees les plus recentes du profil `space`, filtrees par `terms` si la chaine n'est pas vide.
/// Rend aussi le nombre total d'adresses distinctes du profil.
pub fn search(library: &Library, terms: &str, space: &str) -> (Vec<Entry>, usize) {
    library
        .with(|db| {
            let total: usize = db.query_row(
                "SELECT COUNT(DISTINCT url) FROM history WHERE space = ?1", params![space], |row| row.get(0),
            )?;
            let entries = if terms.trim().is_empty() {
                latest_in(db, PAGE, Some(space))?
            } else {
                matching(db, terms.trim(), space)?
            };
            Ok((entries, total))
        })
        .unwrap_or_default()
}

/// Une entree par adresse (et par profil), sa derniere visite ; `space` : un seul profil, sinon tous.
fn latest_in(db: &Connection, limit: usize, space: Option<&str>) -> rusqlite::Result<Vec<Entry>> {
    let mut statement = db.prepare(
        "SELECT url, title, favicon, MAX(visited_at) AS seen, COUNT(*) AS visits, space
         FROM history WHERE ?2 IS NULL OR space = ?2 GROUP BY url, space ORDER BY seen DESC LIMIT ?1",
    )?;
    collect(statement.query_map(params![limit as i64, space], row_to_entry)?)
}

fn matching(db: &Connection, terms: &str, space: &str) -> rusqlite::Result<Vec<Entry>> {
    let pattern = format!("%{}%", terms.replace('%', "\\%"));
    let mut statement = db.prepare(
        "SELECT url, title, favicon, MAX(visited_at) AS seen, COUNT(*) AS visits, space
         FROM history WHERE (url LIKE ?1 ESCAPE '\\' OR title LIKE ?1 ESCAPE '\\') AND space = ?3
         GROUP BY url ORDER BY seen DESC LIMIT ?2",
    )?;
    collect(statement.query_map(params![pattern, PAGE as i64, space], row_to_entry)?)
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    Ok(Entry {
        url: row.get(0)?,
        title: row.get(1)?,
        favicon: row.get(2)?,
        visited_at: row.get(3)?,
        visits: row.get::<_, i64>(4)? as u32,
        space: row.get(5)?,
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

/// Efface l'historique du profil `space` depuis `since` (secondes ; 0 = tout). Chaque adresse effacee est notee
/// pour les autres machines du compte. Rend le nombre d'adresses effacees.
pub fn forget_since(library: &Library, since: i64, space: &str) -> usize {
    let urls: Vec<String> = library
        .with(|db| {
            let mut statement =
                db.prepare("SELECT DISTINCT url FROM history WHERE space = ?1 AND visited_at >= ?2")?;
            let rows = statement.query_map(params![space, since], |row| row.get::<_, String>(0))?;
            rows.collect()
        })
        .unwrap_or_default();
    let at = now();
    for url in &urls {
        forget(library, url, at);
    }
    urls.len()
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

/// Les `limit` adresses visitees le plus recemment, tous profils (synchronisation).
pub fn latest(library: &Library, limit: usize) -> Vec<Entry> {
    library.with(|db| latest_in(db, limit, None)).unwrap_or_default()
}

/// Ajoute une visite venue d'une autre machine (sans doublon), sauf si elle a ete effacee ici depuis.
pub fn import(library: &Library, url: &str, title: &str, visited_at: i64, space: &str) -> bool {
    library
        .with(|db| {
            db.execute(
                "INSERT OR IGNORE INTO history (url, title, favicon, visited_at, space)
                 SELECT ?1, ?2, NULL, ?3, ?4
                 WHERE NOT EXISTS (SELECT 1 FROM history_forgotten WHERE url IN (?1, '*') AND at >= ?3)",
                params![url, title, visited_at, space],
            )
        })
        .is_some()
}
