//! Historique synchronise (reglage `sync.history`, actif par defaut) : les adresses visitees le plus recemment et les
//! effacements recents partent dans le coffre ; celles des autres machines rejoignent l'historique local. Les adresses
//! trop longues restent sur la machine (le coffre est plafonne a 1 Mo par type).

use echo_account::sync::HISTORY_MAX;
use echo_library::history;
use echo_library::settings::Value as Setting;
use serde_json::{json, Map, Value};

pub const SETTING: &str = "sync.history";
const MAX_URL: usize = 400;
const MAX_TITLE: usize = 80;

pub fn enabled() -> bool {
    crate::session::with(|s| {
        echo_library::settings::all(&s.library).into_iter().any(|(k, v)| k == SETTING && v == Setting::Flag(true))
    })
    .unwrap_or(false)
}

/// La valeur locale : `{adresse: {"t", "v"}}` pour les visites, `{adresse | "*": {"d"}}` pour les effacements.
pub fn local_value() -> Value {
    let mut all = Map::new();
    crate::session::with(|s| {
        for (url, at) in history::forgotten(&s.library) {
            all.insert(url, json!({"d": at}));
        }
        for entry in history::latest(&s.library, HISTORY_MAX) {
            let forgotten_after = all.get(&entry.url).and_then(|e| e["d"].as_i64()).is_some_and(|d| d >= entry.visited_at);
            if entry.url.len() <= MAX_URL && !forgotten_after {
                let title: String = entry.title.chars().take(MAX_TITLE).collect();
                all.insert(entry.url, json!({"t": title, "v": entry.visited_at}));
            }
        }
    });
    Value::Object(all)
}

/// Ecrit la version fusionnee : visites venues d'ailleurs ajoutees, effacements faits ailleurs appliques.
pub fn apply(merged: &Value) {
    let Some(entries) = merged.as_object() else { return };
    crate::session::with(|s| {
        for (url, entry) in entries {
            match (entry["v"].as_i64(), entry["d"].as_i64()) {
                (Some(at), _) if url != history::ALL => {
                    history::import(&s.library, url, entry["t"].as_str().unwrap_or_default(), at);
                }
                (_, Some(at)) => {
                    history::forget(&s.library, url, at);
                }
                _ => {}
            }
        }
    });
    crate::bridge::publish_history("");
}
