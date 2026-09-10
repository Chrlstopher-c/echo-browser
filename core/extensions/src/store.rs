//! Responsabilite : l'inventaire des extensions installees, sur disque et en memoire.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::warn;

/// Une extension telle que le navigateur la connait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    /// Identifiant du catalogue Chrome, 32 lettres.
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    /// Vrai si l'utilisateur peut la retirer — les pieces internes ne le sont pas.
    #[serde(default)]
    pub removable: bool,
    /// Vrai si elle est chargee par la ligne de commande : la retirer demande une relance.
    #[serde(default)]
    pub from_command_line: bool,
}

/// Emplacement d'une extension depaquetee.
pub fn extension_dir(root: &Path, id: &str) -> PathBuf {
    root.join(id)
}

/// Lit le manifeste d'une extension depaquetee.
pub fn read_manifest(dir: &Path) -> Option<(String, String)> {
    let text = std::fs::read_to_string(dir.join("manifest.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    let name = value.get("name")?.as_str()?.to_string();
    let version = value.get("version")?.as_str().unwrap_or("0").to_string();
    Some((name, version))
}

/// Le nom lisible d'une extension. Les paquets traduits stockent un jeton `__MSG_...__`
/// qu'il faut resoudre dans les fichiers de langue.
pub fn resolve_name(dir: &Path, raw: &str) -> String {
    let Some(key) = raw.strip_prefix("__MSG_").and_then(|r| r.strip_suffix("__")) else {
        return raw.to_string();
    };
    for locale in ["fr", "fr_FR", "en", "en_US"] {
        let path = dir.join("_locales").join(locale).join("messages.json");
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else { continue };
        if let Some(message) = value.get(key).and_then(|m| m.get("message")).and_then(|m| m.as_str()) {
            return message.to_string();
        }
    }
    raw.to_string()
}

/// Fichier qui retient l'etat d'activation, que le manifeste ne porte pas.
fn state_path(root: &Path) -> PathBuf {
    root.join("state.json")
}

/// Recense les extensions presentes sur disque.
pub fn list(root: &Path) -> Vec<Extension> {
    let disabled = read_disabled(root);
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut found: Vec<Extension> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|path| {
            let id = path.file_name()?.to_str()?.to_string();
            let (raw_name, version) = read_manifest(&path)?;
            Some(Extension {
                enabled: !disabled.contains(&id),
                name: resolve_name(&path, &raw_name),
                version,
                id,
                removable: true,
                from_command_line: true,
            })
        })
        .collect();
    found.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    found
}

/// Bascule l'activation d'une extension. Le changement prend effet au prochain demarrage.
pub fn set_enabled(root: &Path, id: &str, enabled: bool) -> anyhow::Result<()> {
    let mut disabled = read_disabled(root);
    if enabled {
        disabled.retain(|entry| entry != id);
    } else if !disabled.iter().any(|entry| entry == id) {
        disabled.push(id.to_string());
    }
    std::fs::create_dir_all(root)?;
    std::fs::write(state_path(root), serde_json::to_vec_pretty(&disabled)?)?;
    Ok(())
}

fn read_disabled(root: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(state_path(root)) else {
        return Vec::new();
    };
    serde_json::from_str(&text).unwrap_or_else(|err| {
        warn!(%err, "etat des extensions illisible");
        Vec::new()
    })
}

/// Supprime une extension du disque.
pub fn remove(root: &Path, id: &str) -> anyhow::Result<()> {
    let dir = extension_dir(root, id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    set_enabled(root, id, true)?;
    Ok(())
}
