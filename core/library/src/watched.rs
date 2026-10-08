//! Responsabilite : les pages surveillees — le texte vu a la derniere visite, et ce qui a change depuis (lignes ajoutees
//! et retirees). Rien ne part du navigateur : la comparaison se fait ici, a chaque visite.

use std::collections::HashSet;

use rusqlite::{params, OptionalExtension};

use crate::{now, Library};

/// Lignes montrees au plus de chaque cote.
const SHOWN: usize = 20;
const MAX_TEXT: usize = 60_000;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Change {
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

pub fn watch(library: &Library, url: &str) -> bool {
    library
        .with(|db| {
            db.execute("INSERT OR IGNORE INTO watched_pages (url, text, checked) VALUES (?1, NULL, ?2)",
                params![url, now()])
        })
        .is_some()
}

pub fn unwatch(library: &Library, url: &str) -> bool {
    library.with(|db| db.execute("DELETE FROM watched_pages WHERE url = ?1", params![url])).is_some()
}

pub fn is_watched(library: &Library, url: &str) -> bool {
    library
        .with(|db| db.query_row("SELECT 1 FROM watched_pages WHERE url = ?1", params![url], |_| Ok(())).optional())
        .flatten()
        .is_some()
}

fn lines(text: &str) -> Vec<String> {
    text.lines().map(|l| l.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|l| l.chars().count() > 2).collect()
}

/// Les lignes de `new` absentes de `old` (dans l'ordre de `new`), au plus `SHOWN`.
fn missing(new: &[String], old: &[String]) -> Vec<String> {
    let known: HashSet<&String> = old.iter().collect();
    let mut seen = HashSet::new();
    new.iter().filter(|l| !known.contains(l) && seen.insert(*l)).take(SHOWN).cloned().collect()
}

/// Enregistre le texte de la visite ; rend le changement depuis la precedente (rien a la premiere, ni si identique).
pub fn visited(library: &Library, url: &str, text: &str) -> Option<Change> {
    let text: String = text.chars().take(MAX_TEXT).collect();
    let previous: Option<Option<String>> = library
        .with(|db| {
            db.query_row("SELECT text FROM watched_pages WHERE url = ?1", params![url], |row| row.get(0)).optional()
        })
        .flatten();
    let previous = previous?;
    library.with(|db| {
        db.execute("UPDATE watched_pages SET text = ?2, checked = ?3 WHERE url = ?1", params![url, text, now()])
    })?;
    let (old, new) = (lines(&previous?), lines(&text));
    let change = Change { added: missing(&new, &old), removed: missing(&old, &new) };
    (!change.added.is_empty() || !change.removed.is_empty()).then_some(change)
}
