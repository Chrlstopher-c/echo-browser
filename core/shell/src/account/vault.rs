//! Ce que le service garde du compte, rendu lisible : chaque type est dechiffre sur la machine et resume en lignes.
//! Le service lui-meme ne voit que des donnees chiffrees.

use echo_account::api::{Item, Session};
use echo_contract::{VaultKindView, VaultLineView};
use serde_json::Value;

const MAX_LINES: usize = 300;

fn line(title: impl Into<String>, detail: impl Into<String>) -> VaultLineView {
    VaultLineView { title: title.into(), detail: detail.into() }
}

fn lines_of(kind: &str, value: &Value) -> Vec<VaultLineView> {
    let empty = serde_json::Map::new();
    let object = value.as_object().unwrap_or(&empty);
    match kind {
        "favoris" => value
            .as_array()
            .map(|all| all.iter().map(|f| line(f["title"].as_str().unwrap_or_default(), f["url"].as_str().unwrap_or_default())).collect())
            .unwrap_or_default(),
        "extensions" => object
            .iter()
            .map(|(profile, ids)| line(format!("Profil {profile}"), format!("{} extension(s)", ids.as_array().map_or(0, Vec::len))))
            .collect(),
        "onglets" => object
            .values()
            .map(|m| line(m["nom"].as_str().unwrap_or("Machine"), format!("{} onglet(s)", m["onglets"].as_array().map_or(0, Vec::len))))
            .collect(),
        "historique" => history_lines(object),
        _ => object.iter().map(|(key, v)| line(key.clone(), v.to_string())).collect(),
    }
}

/// Visites d'abord (les plus recentes en tete), puis les effacements.
fn history_lines(entries: &serde_json::Map<String, Value>) -> Vec<VaultLineView> {
    let mut visits: Vec<(&String, &Value)> = entries.iter().filter(|(_, e)| e["v"].is_i64()).collect();
    visits.sort_by_key(|(_, e)| std::cmp::Reverse(e["v"].as_i64()));
    let erased = entries.len() - visits.len();
    let mut out: Vec<VaultLineView> = visits.into_iter().map(|(url, e)| line(e["t"].as_str().unwrap_or_default(), url.clone())).collect();
    if erased > 0 {
        out.push(line("Effacements", format!("{erased} adresse(s) effacée(s), transmis aux autres machines")));
    }
    out
}

fn count_of(value: &Value) -> usize {
    value.as_array().map(Vec::len).or_else(|| value.as_object().map(serde_json::Map::len)).unwrap_or(0)
}

pub fn view(session: &Session, items: &[Item]) -> Vec<VaultKindView> {
    items
        .iter()
        .map(|item| {
            let value = echo_account::crypto::open(&session.encryption, &item.kind, &item.data)
                .ok()
                .and_then(|plain| serde_json::from_slice::<Value>(&plain).ok())
                .unwrap_or(Value::Null);
            let mut lines = lines_of(&item.kind, &value);
            lines.truncate(MAX_LINES);
            VaultKindView {
                kind: item.kind.clone(),
                version: item.version,
                bytes: item.data.len() as u64,
                updated: item.updated,
                count: count_of(&value),
                lines,
            }
        })
        .collect()
}
