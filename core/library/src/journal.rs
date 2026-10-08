//! Responsabilite : le journal d'acces de chaque site — ce qu'il a fait de sensible (premier contact avec un tiers,
//! permission demandee et la decision, telechargement). 200 entrees gardees par site.

use rusqlite::params;

use crate::{now, Library};

const KEPT_PER_SITE: i64 = 200;

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub at: i64,
    /// `tiers`, `permission`, `telechargement`.
    pub kind: String,
    pub detail: String,
}

/// Note un acces. Avec `once`, rien n'est note si la meme entree (type et detail) existe deja pour ce site.
pub fn record(library: &Library, site: &str, kind: &str, detail: &str, once: bool) -> bool {
    library
        .with(|db| {
            let written = db.execute(
                "INSERT INTO site_journal (site, at, kind, detail) SELECT ?1, ?2, ?3, ?4
                 WHERE NOT ?5 OR NOT EXISTS (SELECT 1 FROM site_journal WHERE site = ?1 AND kind = ?3 AND detail = ?4)",
                params![site, now(), kind, detail, once],
            )?;
            db.execute(
                "DELETE FROM site_journal WHERE site = ?1 AND rowid NOT IN
                 (SELECT rowid FROM site_journal WHERE site = ?1 ORDER BY at DESC, rowid DESC LIMIT ?2)",
                params![site, KEPT_PER_SITE],
            )?;
            Ok(written > 0)
        })
        .unwrap_or(false)
}

/// Les entrees d'un site, les plus recentes d'abord.
pub fn list(library: &Library, site: &str, limit: usize) -> Vec<Entry> {
    library
        .with(|db| {
            let mut statement = db.prepare(
                "SELECT at, kind, detail FROM site_journal WHERE site = ?1 ORDER BY at DESC, rowid DESC LIMIT ?2",
            )?;
            let rows = statement.query_map(params![site, limit as i64], |row| {
                Ok(Entry { at: row.get(0)?, kind: row.get(1)?, detail: row.get(2)? })
            })?;
            rows.collect()
        })
        .unwrap_or_default()
}

pub fn clear(library: &Library, site: &str) -> bool {
    library.with(|db| db.execute("DELETE FROM site_journal WHERE site = ?1", params![site])).is_some()
}
