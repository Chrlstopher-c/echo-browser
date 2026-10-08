//! Responsabilite : les signaux anonymes en attente d'envoi — comptes par jour de domaines visites et d'hotes bloques,
//! tenus seulement si l'utilisateur a choisi de partager. Effaces des qu'ils sont partis (ou trop vieux).

use rusqlite::params;

use crate::Library;

/// Ajoute des comptes a un jour (`kind` : `sites` ou `bloques`).
pub fn add(library: &Library, day: &str, counts: &[(String, String, u32)]) -> bool {
    library
        .with(|db| {
            let mut statement = db.prepare(
                "INSERT INTO signals_day (day, kind, key, n) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (day, kind, key) DO UPDATE SET n = n + excluded.n",
            )?;
            for (kind, key, n) in counts {
                statement.execute(params![day, kind, key, n])?;
            }
            Ok(())
        })
        .is_some()
}

/// Les jours en attente, du plus ancien au plus recent.
pub fn days(library: &Library) -> Vec<String> {
    library
        .with(|db| {
            let mut statement = db.prepare("SELECT DISTINCT day FROM signals_day ORDER BY day")?;
            let rows = statement.query_map([], |row| row.get(0))?;
            rows.collect()
        })
        .unwrap_or_default()
}

/// Les `limit` cles les plus comptees d'un jour et d'un type.
pub fn top(library: &Library, day: &str, kind: &str, limit: usize) -> Vec<(String, u32)> {
    library
        .with(|db| {
            let mut statement =
                db.prepare("SELECT key, n FROM signals_day WHERE day = ?1 AND kind = ?2 ORDER BY n DESC LIMIT ?3")?;
            let rows = statement.query_map(params![day, kind, limit as i64], |row| Ok((row.get(0)?, row.get(1)?)))?;
            rows.collect()
        })
        .unwrap_or_default()
}

pub fn forget_day(library: &Library, day: &str) -> bool {
    library.with(|db| db.execute("DELETE FROM signals_day WHERE day = ?1", params![day])).is_some()
}

pub fn clear(library: &Library) -> bool {
    library.with(|db| db.execute("DELETE FROM signals_day", [])).is_some()
}
