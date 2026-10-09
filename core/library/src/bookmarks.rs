//! Responsabilite : les favoris, et l'ordre dans lequel l'utilisateur les a ranges.

use crate::{now, Library};
use rusqlite::{params, Connection};

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub added_at: i64,
    /// Profil auquel le favori appartient.
    pub space: String,
}

/// Ajoute un favori au profil `space`, ou met a jour son titre (et le rattache a ce profil) s'il existe deja.
pub fn add(library: &Library, url: &str, title: &str, favicon: Option<&str>, space: &str) -> bool {
    library
        .with(|db| {
            let position: i64 = db
                .query_row("SELECT COALESCE(MAX(position), -1) + 1 FROM bookmarks", [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);
            db.execute(
                "INSERT INTO bookmarks (url, title, favicon, added_at, position, space)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(url) DO UPDATE SET title = ?2, favicon = ?3, space = ?6",
                params![url, title, favicon, now(), position, space],
            )
        })
        .is_some()
}

pub fn remove(library: &Library, url: &str) -> bool {
    library
        .with(|db| db.execute("DELETE FROM bookmarks WHERE url = ?1", params![url]))
        .is_some()
}

/// Deplace un favori a la place `to` de la liste du profil `space` ; les favoris des autres profils gardent leurs places.
pub fn move_to(library: &Library, url: &str, to: usize, space: &str) -> bool {
    library
        .with(|db| {
            let mine = read_order(db, space)?;
            let slots: Vec<i64> = mine.iter().map(|(_, position)| *position).collect();
            let mut urls: Vec<String> = mine.into_iter().map(|(url, _)| url).collect();
            let Some(from) = urls.iter().position(|entry| entry == url) else {
                return Ok(0);
            };
            let entry = urls.remove(from);
            urls.insert(to.min(urls.len()), entry);
            for (url, slot) in urls.iter().zip(&slots) {
                db.execute("UPDATE bookmarks SET position = ?1 WHERE url = ?2", params![slot, url])?;
            }
            Ok(urls.len())
        })
        .is_some()
}

fn read_order(db: &Connection, space: &str) -> rusqlite::Result<Vec<(String, i64)>> {
    let mut statement = db.prepare("SELECT url, position FROM bookmarks WHERE space = ?1 ORDER BY position ASC")?;
    let rows = statement.query_map(params![space], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))?;
    rows.collect()
}

/// Tous les favoris, de tous les profils (synchronisation).
pub fn list(library: &Library) -> Vec<Bookmark> {
    library
        .with(|db| {
            let mut statement = db.prepare(
                "SELECT url, title, favicon, added_at, space FROM bookmarks ORDER BY position ASC",
            )?;
            let rows = statement.query_map([], |row| {
                Ok(Bookmark {
                    url: row.get(0)?,
                    title: row.get(1)?,
                    favicon: row.get(2)?,
                    added_at: row.get(3)?,
                    space: row.get(4)?,
                })
            })?;
            rows.collect()
        })
        .unwrap_or_default()
}

/// Les favoris du profil `space`, dans leur ordre.
pub fn list_in(library: &Library, space: &str) -> Vec<Bookmark> {
    list(library).into_iter().filter(|b| b.space == space).collect()
}

/// Vrai si l'adresse est deja en favori.
pub fn contains(library: &Library, url: &str) -> bool {
    library
        .with(|db| {
            db.query_row("SELECT 1 FROM bookmarks WHERE url = ?1", params![url], |_| Ok(true))
        })
        .unwrap_or(false)
}
