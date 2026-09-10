//! Responsabilite : charger les scriptlets et ressources de remplacement — sans eux, les filtres `+js(...)` ne font rien.

use adblock::resources::Resource;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Nom du paquet genere par `tools/build-resources.mjs`.
pub const PACK_FILE: &str = "shield-resources.json";

pub fn pack_path(data_dir: &Path) -> PathBuf {
    data_dir.join(PACK_FILE)
}

/// Charge le paquet de scriptlets. Un paquet absent n'est pas fatal : le blocage reseau
/// et le masquage fonctionnent sans, mais les contournements d'anti-adblock non.
pub fn load_pack(path: &Path) -> Vec<Resource> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            warn!(?path, %err, "paquet de scriptlets absent — les filtres +js() resteront inertes");
            return Vec::new();
        }
    };
    match serde_json::from_str::<Vec<Resource>>(&text) {
        Ok(resources) => {
            info!(nombre = resources.len(), "scriptlets charges");
            resources
        }
        Err(err) => {
            warn!(?path, %err, "paquet de scriptlets illisible");
            Vec::new()
        }
    }
}
