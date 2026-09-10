//! Responsabilite : lire l'inventaire des extensions tenu par Chromium lui-meme.
//!
//! Chromium installe, met a jour et desinstalle ses extensions dans son profil. Tenir
//! un second inventaire a cote produirait deux verites divergentes — celle du panneau
//! et celle du navigateur. On lit donc la sienne.

use crate::store::Extension;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tracing::warn;

/// Origine d'une extension, telle que Chromium la note dans son profil.
/// Valeurs de `extensions::mojom::ManifestLocation`.
mod location {
    /// Installee depuis le catalogue Chrome.
    pub const STORE: u64 = 1;
    /// Piece interne du navigateur — lecteur PDF et consorts.
    pub const COMPONENT: u64 = 5;
    /// Chargee au demarrage par la ligne de commande.
    pub const COMMAND_LINE: u64 = 8;
}

/// Emplacement du profil par defaut a l'interieur du repertoire de travail.
pub fn default_profile(data_dir: &Path) -> PathBuf {
    data_dir.join("profile").join("Default")
}

/// Recense les extensions visibles par l'utilisateur. Les pieces internes du
/// navigateur sont ecartees : il ne peut ni les retirer, ni les desactiver.
pub fn list(profile: &Path) -> Vec<Extension> {
    let Some(settings) = read_settings(profile) else {
        return Vec::new();
    };
    let mut found: Vec<Extension> = settings
        .iter()
        .filter_map(|(id, entry)| read_entry(profile, id, entry))
        .collect();
    found.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    found
}

fn read_settings(profile: &Path) -> Option<serde_json::Map<String, Value>> {
    let text = std::fs::read_to_string(profile.join("Preferences")).ok()?;
    let value: Value = serde_json::from_str(&text)
        .inspect_err(|err| warn!(%err, "preferences du profil illisibles"))
        .ok()?;
    value
        .get("extensions")?
        .get("settings")?
        .as_object()
        .cloned()
}

fn read_entry(profile: &Path, id: &str, entry: &Value) -> Option<Extension> {
    let origin = entry.get("location").and_then(Value::as_u64).unwrap_or(0);
    if origin == location::COMPONENT {
        return None;
    }
    let manifest = entry.get("manifest")?;
    let name = manifest.get("name")?.as_str()?.to_string();
    let version = manifest
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("0")
        .to_string();

    // Chromium n'ecrit `state` que lorsqu'il a ete change : son absence vaut « active ».
    let enabled = entry.get("state").and_then(Value::as_u64).unwrap_or(1) == 1;
    let dir = entry_dir(profile, id, entry);
    let name = resolve_name(&dir, &name);

    Some(Extension {
        id: id.to_string(),
        name,
        version,
        enabled,
        removable: origin == location::STORE,
        from_command_line: origin == location::COMMAND_LINE,
        action: crate::action::Action::from_manifest(manifest),
        description: crate::action::description(manifest, &dir),
        permissions: crate::action::permissions(manifest),
        dir,
    })
}

/// Dossier de l'extension : relatif au profil pour celles du catalogue, absolu sinon.
fn entry_dir(profile: &Path, id: &str, entry: &Value) -> PathBuf {
    let path = entry.get("path").and_then(Value::as_str).unwrap_or(id);
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        candidate
    } else {
        profile.join("Extensions").join(candidate)
    }
}

/// Les paquets traduits stockent un jeton `__MSG_...__` a resoudre dans les langues.
fn resolve_name(dir: &Path, raw: &str) -> String {
    let Some(key) = raw.strip_prefix("__MSG_").and_then(|r| r.strip_suffix("__")) else {
        return raw.to_string();
    };
    for locale in ["fr", "fr_FR", "en", "en_US"] {
        let path = dir.join("_locales").join(locale).join("messages.json");
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let Ok(value) = serde_json::from_str::<Value>(&text) else { continue };
        if let Some(message) = value.get(key).and_then(|m| m.get("message")).and_then(Value::as_str)
        {
            return message.to_string();
        }
    }
    raw.to_string()
}

/// Adresse de la fiche d'une extension dans le catalogue.
pub fn store_page(id: &str) -> String {
    format!("https://chromewebstore.google.com/detail/{id}")
}

/// Page de gestion des extensions, servie par Chromium.
pub const MANAGE_PAGE: &str = "chrome://extensions";
