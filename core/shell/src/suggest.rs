//! Responsabilite : les suggestions de la page « nouvel onglet » — onglets ouverts, favoris et
//! historique correspondant a ce que l'utilisateur tape.

use serde_json::{json, Value};

const PER_SECTION: usize = 6;

fn matches(text: &str, needle: &str) -> bool {
    needle.is_empty() || text.to_lowercase().contains(needle)
}

/// A appeler sur le thread interface.
pub fn suggest(query: &str) -> Value {
    let needle = query.trim().to_lowercase();
    let found = crate::session::with(|s| {
        let tabs: Vec<Value> = s
            .tabs
            .snapshot()
            .iter()
            .filter(|t| !t.url.starts_with("echo://") && (matches(&t.title, &needle) || matches(&t.url, &needle)))
            .take(PER_SECTION)
            .map(|t| json!({"id": t.id, "title": t.title, "url": t.url}))
            .collect();
        let bookmarks: Vec<Value> = echo_library::bookmarks::list(&s.library)
            .into_iter()
            .filter(|b| matches(&b.title, &needle) || matches(&b.url, &needle))
            .take(PER_SECTION)
            .map(|b| json!({"title": b.title, "url": b.url}))
            .collect();
        let mut seen = std::collections::HashSet::new();
        let history: Vec<Value> = echo_library::history::search(&s.library, query)
            .0
            .into_iter()
            .filter(|e| seen.insert(e.url.trim_end_matches('/').to_string()))
            .take(PER_SECTION)
            .map(|e| json!({"title": e.title, "url": e.url}))
            .collect();
        json!({"ok": true, "tabs": tabs, "bookmarks": bookmarks, "history": history})
    });
    found.unwrap_or_else(|| json!({"ok": false, "error": "navigateur occupe"}))
}
