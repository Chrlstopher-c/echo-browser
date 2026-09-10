//! Responsabilite : les reglages du navigateur, et leurs valeurs par defaut.

use crate::Library;
use rusqlite::params;

/// Un reglage : sa cle, et ce qu'il vaut.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Flag(bool),
    Text(String),
    Number(f64),
}

impl Value {
    fn encode(&self) -> String {
        match self {
            Value::Flag(on) => format!("b:{on}"),
            Value::Text(text) => format!("s:{text}"),
            Value::Number(number) => format!("n:{number}"),
        }
    }

    fn decode(raw: &str) -> Option<Self> {
        let (kind, rest) = raw.split_once(':')?;
        match kind {
            "b" => Some(Value::Flag(rest == "true")),
            "s" => Some(Value::Text(rest.to_string())),
            "n" => rest.parse().ok().map(Value::Number),
            _ => None,
        }
    }
}

/// Les reglages du navigateur et leur valeur de depart.
pub fn defaults() -> Vec<(&'static str, Value)> {
    vec![
        ("shield.enabled", Value::Flag(true)),
        ("shield.strict", Value::Flag(false)),
        ("search.engine", Value::Text("google".into())),
        ("startup.restore_tabs", Value::Flag(true)),
        ("downloads.ask_location", Value::Flag(false)),
        ("appearance.space", Value::Text("graphite".into())),
        ("appearance.sidebar_width", Value::Number(240.0)),
        ("privacy.send_do_not_track", Value::Flag(true)),
        ("privacy.clear_on_exit", Value::Flag(false)),
    ]
}

/// Tous les reglages : ceux enregistres, completes par les valeurs par defaut.
pub fn all(library: &Library) -> Vec<(String, Value)> {
    let stored = library
        .with(|db| {
            let mut statement = db.prepare("SELECT key, value FROM settings")?;
            let rows = statement.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap_or_default();

    defaults()
        .into_iter()
        .map(|(key, fallback)| {
            let value = stored
                .iter()
                .find(|(stored_key, _)| stored_key == key)
                .and_then(|(_, raw)| Value::decode(raw))
                .unwrap_or(fallback);
            (key.to_string(), value)
        })
        .collect()
}

/// Enregistre un reglage. Refuse une cle inconnue : l'interface ne doit pas
/// pouvoir inventer des reglages que le navigateur n'honore pas.
pub fn set(library: &Library, key: &str, value: &Value) -> Result<(), String> {
    if !defaults().iter().any(|(known, _)| *known == key) {
        return Err(format!(
            "reglage inconnu « {key} ». Attendus : {}",
            defaults().iter().map(|(k, _)| *k).collect::<Vec<_>>().join(", ")
        ));
    }
    library
        .with(|db| {
            db.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = ?2",
                params![key, value.encode()],
            )
        })
        .map(|_| ())
        .ok_or_else(|| "enregistrement impossible".to_string())
}
