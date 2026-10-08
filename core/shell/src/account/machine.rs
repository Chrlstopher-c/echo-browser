//! Onglets des autres machines : cette machine publie ses onglets ouverts sous son identifiant (type `onglets` du coffre,
//! une entree par machine) et montre ceux des autres. Seules les pages web partent (pas les pages d'Echo), 100 au plus.

use std::path::PathBuf;

use echo_contract::{CoreEvent, RemoteMachineView, RemoteTabView};
use serde_json::{json, Map, Value};

const MAX_TABS: usize = 100;
const MAX_TITLE: usize = 200;

fn id_path() -> PathBuf {
    crate::flags::data_dir().join("machine-id")
}

/// Identifiant stable de cette machine pour le compte (aleatoire, cree au premier usage).
pub fn id() -> String {
    if let Ok(known) = std::fs::read_to_string(id_path()) {
        let known = known.trim();
        if !known.is_empty() {
            return known.to_string();
        }
    }
    let fresh: String = echo_account::crypto::random_bytes::<16>()
        .map(|b| b.iter().map(|x| format!("{x:02x}")).collect())
        .unwrap_or_else(|_| "machine".to_string());
    let _ = std::fs::write(id_path(), &fresh);
    fresh
}

fn name() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|n| n.trim().to_string())
        .ok()
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "Machine sans nom".to_string())
}

/// La valeur locale du type `onglets` : celles des autres machines telles que deja connues, plus la notre a jour.
pub fn local_value(known: Option<&Value>) -> Value {
    let mut all: Map<String, Value> = known.and_then(Value::as_object).cloned().unwrap_or_default();
    let tabs: Vec<Value> = crate::session::with(|s| {
        s.tabs
            .iter()
            .filter(|t| t.url.starts_with("http"))
            .take(MAX_TABS)
            .map(|t| json!({"url": t.url, "title": t.title.chars().take(MAX_TITLE).collect::<String>()}))
            .collect()
    })
    .unwrap_or_default();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    all.insert(id(), json!({"nom": name(), "maj": now, "onglets": tabs}));
    Value::Object(all)
}

/// Diffuse les onglets des autres machines (tout sauf celle-ci), les plus recentes d'abord.
pub fn publish(known: Option<&Value>) {
    let me = id();
    let mut machines: Vec<RemoteMachineView> = known
        .and_then(Value::as_object)
        .map(|all| {
            all.iter()
                .filter(|(key, _)| **key != me)
                .map(|(_, m)| RemoteMachineView {
                    name: m["nom"].as_str().unwrap_or("Machine").to_string(),
                    updated: m["maj"].as_i64().unwrap_or(0),
                    tabs: m["onglets"]
                        .as_array()
                        .map(|tabs| {
                            tabs.iter()
                                .filter_map(|t| Some(RemoteTabView {
                                    url: t["url"].as_str()?.to_string(),
                                    title: t["title"].as_str().unwrap_or_default().to_string(),
                                }))
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .filter(|m| !m.tabs.is_empty())
                .collect()
        })
        .unwrap_or_default();
    machines.sort_by_key(|m| std::cmp::Reverse(m.updated));
    crate::bridge::publish(&CoreEvent::RemoteTabsChanged { machines });
}
