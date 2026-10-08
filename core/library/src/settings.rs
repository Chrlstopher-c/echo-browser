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
/// Sites dont l'onglet ne dort jamais : ils recoivent des messages en arriere-plan.
const NEVER_SLEEP: &str = "mail.google.com,outlook.live.com,outlook.office.com,web.whatsapp.com,discord.com,\
app.slack.com,teams.microsoft.com,messenger.com,web.telegram.org";

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
        ("appearance.shell", Value::Text("#222326".into())),
        ("session.restore", Value::Flag(true)),
        ("tabs.sleepEnabled", Value::Flag(true)),
        ("tabs.sleepAfterMinutes", Value::Number(5.0)),
        ("tabs.folders", Value::Text("[]".into())),
        ("tabs.containers", Value::Text("[]".into())),
        ("profiles.names", Value::Text("{}".into())),
        ("tabs.neverSleep", Value::Text(NEVER_SLEEP.into())),
        ("devtools.width", Value::Number(560.0)),
        ("video.codecsPrompt", Value::Flag(true)),
        ("updates.auto", Value::Flag(true)),
        ("sync.history", Value::Flag(true)),
        ("sync.mode", Value::Text("auto".into())),
        ("signals.share", Value::Flag(false)),
        ("profiles.list", Value::Text(String::new())),
        ("onboarding.done", Value::Flag(false)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reglages_ecrits_par_l_interface_existent() {
        // L'interface ecrit ces cles elle-meme (dossiers, conteneurs, sites eveilles) : absentes, la creation echoue.
        for key in ["tabs.folders", "tabs.containers", "tabs.neverSleep", "shield.enabled"] {
            assert!(defaults().iter().any(|(k, _)| *k == key), "reglage manquant : {key}");
        }
    }
}
