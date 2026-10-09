//! Ce qui se synchronise, lu et ecrit dans la session : reglages (une liste choisie, rien qui depende de la machine),
//! favoris, extensions par profil. Appele sur le fil de l'interface.

use std::collections::{BTreeMap, HashSet};

use echo_library::settings::Value as Setting;
use serde_json::{json, Value};
use tracing::warn;

/// Reglages partages entre machines. Pas la largeur des outils, ni le profil affiche : ils dependent de l'ecran.
const SYNCED: [&str; 17] = [
    "shield.enabled", "shield.strict", "search.engine", "search.suggest", "startup.restore_tabs", "downloads.ask_location",
    "privacy.send_do_not_track", "privacy.clear_on_exit", "session.restore", "tabs.sleepEnabled",
    "tabs.sleepAfterMinutes", "tabs.neverSleep", "tabs.containers", "profiles.names", "profiles.list", "video.codecsPrompt",
    "forms.cards",
];

fn to_json(value: &Setting) -> Value {
    match value {
        Setting::Flag(on) => json!(on),
        Setting::Text(text) => json!(text),
        Setting::Number(n) => json!(n),
    }
}

fn from_json(value: &Value) -> Option<Setting> {
    match value {
        Value::Bool(on) => Some(Setting::Flag(*on)),
        Value::String(text) => Some(Setting::Text(text.clone())),
        Value::Number(n) => n.as_f64().map(Setting::Number),
        _ => None,
    }
}

/// Les valeurs locales de chaque type synchronise.
pub fn read() -> BTreeMap<String, Value> {
    let registry = crate::extension_profiles::registry();
    crate::session::with(|s| {
        let reglages: serde_json::Map<String, Value> = echo_library::settings::all(&s.library)
            .into_iter()
            .filter(|(key, _)| SYNCED.contains(&key.as_str()))
            .map(|(key, value)| (key, to_json(&value)))
            .collect();
        let favoris: Vec<Value> = echo_library::bookmarks::list(&s.library)
            .into_iter()
            .map(|b| json!({"url": b.url, "title": b.title, "space": b.space}))
            .collect();
        BTreeMap::from([
            ("reglages".to_string(), Value::Object(reglages)),
            ("favoris".to_string(), Value::Array(favoris)),
            ("extensions".to_string(), serde_json::to_value(&registry).unwrap_or_default()),
        ])
    })
    .unwrap_or_default()
}

/// Ecrit les valeurs fusionnees recues du service.
pub fn apply(writes: &BTreeMap<String, Value>) {
    if let Some(reglages) = writes.get("reglages").and_then(Value::as_object) {
        crate::session::with(|s| {
            for (key, value) in reglages.iter().filter(|(k, _)| SYNCED.contains(&k.as_str())) {
                if let Some(setting) = from_json(value) {
                    if let Err(err) = echo_library::settings::set(&s.library, key, &setting) {
                        warn!(%key, %err, "reglage synchronise non ecrit");
                    }
                }
            }
        });
        crate::bridge::publish_settings();
    }
    if let Some(favoris) = writes.get("favoris").and_then(Value::as_array) {
        apply_bookmarks(favoris);
    }
    if let Some(extensions) = writes.get("extensions") {
        apply_extensions(extensions);
    }
}

fn apply_bookmarks(favoris: &[Value]) {
    let wanted: Vec<(String, String, String)> = favoris
        .iter()
        .filter_map(|f| {
            let space = f["space"].as_str().unwrap_or(crate::profiles::DEFAULT).to_string();
            Some((f["url"].as_str()?.to_string(), f["title"].as_str().unwrap_or_default().to_string(), space))
        })
        .collect();
    let urls: HashSet<&str> = wanted.iter().map(|(u, _, _)| u.as_str()).collect();
    crate::session::with(|s| {
        for bookmark in echo_library::bookmarks::list(&s.library) {
            if !urls.contains(bookmark.url.as_str()) {
                echo_library::bookmarks::remove(&s.library, &bookmark.url);
            }
        }
        let known: Vec<(String, String)> =
            echo_library::bookmarks::list(&s.library).into_iter().map(|b| (b.url, b.space)).collect();
        for (url, title, space) in &wanted {
            if !known.iter().any(|(u, sp)| u == url && sp == space) {
                echo_library::bookmarks::add(&s.library, url, title, None, space);
            }
        }
    });
    crate::bridge::publish_bookmarks();
}

/// Extensions par profil : le registre suit ; une extension encore inconnue de cette machine y est declaree (installee
/// par Chromium au prochain demarrage).
fn apply_extensions(extensions: &Value) {
    let Ok(registry) = serde_json::from_value::<echo_extensions::profiles::Registry>(extensions.clone()) else { return };
    let known: HashSet<String> = crate::session::with(|s| s.extensions.list().into_iter().map(|e| e.id).collect())
        .unwrap_or_default();
    for id in registry.values().flatten().filter(|id| !known.contains(*id)) {
        if let Some(Err(err)) = crate::session::with(|s| s.extensions.install(id)) {
            warn!(%id, %err, "extension synchronisee non declaree");
        }
    }
    crate::extension_profiles::save(&registry);
    crate::bridge::publish_extensions();
}
