//! Responsabilite : les favoris, et l'ordre dans lequel l'utilisateur les a ranges.

use crate::{now, Library};
use rusqlite::{params, Connection};

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub added_at: i64,
}

/// Ajoute un favori, ou met a jour son titre s'il existe deja.
pub fn add(library: &Library, url: &str, title: &str, favicon: Option<&str>) -> bool {
    library
        .with(|db| {
            let position: i64 = db
                .query_row("SELECT COALESCE(MAX(position), -1) + 1 FROM bookmarks", [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);
            db.execute(
                "INSERT INTO bookmarks (url, title, favicon, added_at, position)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(url) DO UPDATE SET title = ?2, favicon = ?3",
                params![url, title, favicon, now(), position],
            )
        })
        .is_some()
}

pub fn remove(library: &Library, url: &str) -> bool {
    library
        .with(|db| db.execute("DELETE FROM bookmarks WHERE url = ?1", params![url]))
        .is_some()
}

/// Deplace un favori a une nouvelle position, et renumerote les autres.
pub fn move_to(library: &Library, url: &str, to: usize) -> bool {
    library
        .with(|db| {
            let mut urls = read_order(db)?;
            let Some(from) = urls.iter().position(|entry| entry == url) else {
                return Ok(0);
            };
            let entry = urls.remove(from);
            urls.insert(to.min(urls.len()), entry);
            for (index, url) in urls.iter().enumerate() {
                db.execute(
                    "UPDATE bookmarks SET position = ?1 WHERE url = ?2",
                    params![index as i64, url],
                )?;
            }
            Ok(urls.len())
        })
        .is_some()
}

fn read_order(db: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut statement = db.prepare("SELECT url FROM bookmarks ORDER BY position ASC")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect()
}

pub fn list(library: &Library) -> Vec<Bookmark> {
    library
        .with(|db| {
            let mut statement = db.prepare(
                "SELECT url, title, favicon, added_at FROM bookmarks ORDER BY position ASC",
            )?;
            let rows = statement.query_map([], |row| {
                Ok(Bookmark {
                    url: row.get(0)?,
                    title: row.get(1)?,
                    favicon: row.get(2)?,
                    added_at: row.get(3)?,
                })
            })?;
            rows.collect()
        })
        .unwrap_or_default()
}

/// Vrai si l'adresse est deja en favori.
pub fn contains(library: &Library, url: &str) -> bool {
    library
        .with(|db| {
            db.query_row("SELECT 1 FROM bookmarks WHERE url = ?1", params![url], |_| Ok(true))
        })
        .unwrap_or(false)
}
