//! Responsabilite : les decisions de permission (camera, micro, position, notifications…) retenues
//! par site, pour ne pas reposer la question a chaque visite.

use crate::Library;
use rusqlite::{params, OptionalExtension};

/// La decision retenue pour ce site et cette permission, s'il y en a une.
pub fn get(library: &Library, origin: &str, kind: &str) -> Option<bool> {
    library
        .with(|db| {
            db.query_row(
                "SELECT allow FROM permissions WHERE origin = ?1 AND kind = ?2",
                params![origin, kind],
                |row| row.get::<_, bool>(0),
            )
            .optional()
        })
        .flatten()
}

/// Retient une decision.
pub fn set(library: &Library, origin: &str, kind: &str, allow: bool) -> bool {
    library
        .with(|db| {
            db.execute(
                "INSERT OR REPLACE INTO permissions (origin, kind, allow) VALUES (?1, ?2, ?3)",
                params![origin, kind, allow],
            )
        })
        .is_some()
}

/// Toutes les decisions retenues : (site, permission, autorisee).
pub fn list(library: &Library) -> Vec<(String, String, bool)> {
    library
        .with(|db| {
            let mut stmt = db.prepare("SELECT origin, kind, allow FROM permissions ORDER BY origin, kind")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default()
}

/// Oublie une decision : la question sera reposee a la prochaine demande.
pub fn forget(library: &Library, origin: &str, kind: &str) -> bool {
    library
        .with(|db| db.execute("DELETE FROM permissions WHERE origin = ?1 AND kind = ?2", params![origin, kind]))
        .is_some()
}

/// Oublie toutes les decisions d'un site.
pub fn forget_site(library: &Library, origin: &str) -> bool {
    library.with(|db| db.execute("DELETE FROM permissions WHERE origin = ?1", params![origin])).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library() -> Library {
        Library::in_memory().expect("base en memoire")
    }

    #[test]
    fn retient_et_oublie_une_decision() {
        let lib = library();
        assert_eq!(get(&lib, "https://a.test", "camera"), None);
        assert!(set(&lib, "https://a.test", "camera", true));
        assert_eq!(get(&lib, "https://a.test", "camera"), Some(true));
        assert!(set(&lib, "https://a.test", "camera", false));
        assert_eq!(get(&lib, "https://a.test", "camera"), Some(false));
        assert_eq!(list(&lib), vec![("https://a.test".into(), "camera".into(), false)]);
        assert!(forget(&lib, "https://a.test", "camera"));
        assert!(list(&lib).is_empty());
        assert!(set(&lib, "https://a.test", "mic", true));
        assert!(forget_site(&lib, "https://a.test"));
        assert_eq!(get(&lib, "https://a.test", "camera"), None);
    }
}
