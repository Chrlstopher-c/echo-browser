//! Responsabilite : effacer les traces de navigation — au demarrage si « Tout effacer à la fermeture » est coche
//! (les fichiers de Chromium ne s'effacent surement qu'avant son lancement), et a la demande.

use std::path::Path;
use tracing::{info, warn};

/// Fichiers et dossiers de Chromium qui portent des traces de sites : cookies, caches, stockages de session.
/// Le stockage local reste : l'interface d'Echo y garde ses choix (theme, espace).
const TRACES: &[&str] = &[
    "Cookies", "Cookies-journal", "Cache", "Code Cache", "GPUCache", "IndexedDB", "Session Storage",
    "Shared Dictionary", "DawnGraphiteCache", "DawnWebGPUCache",
];

/// A appeler avant l'initialisation de Chromium.
pub fn wipe_if_asked() {
    let dir = crate::flags::data_dir();
    let Ok(library) = echo_library::Library::open(&dir) else { return };
    let asked = matches!(
        echo_library::settings::get(&library, "privacy.clear_on_exit"),
        Some(echo_library::settings::Value::Flag(true))
    ) || std::fs::remove_file(dir.join(PENDING)).is_ok();
    if !asked {
        return;
    }
    echo_library::history::clear(&library);
    let removed = wipe_traces(&dir.join("profile"), 0);
    info!(removed, "traces de navigation effacees au demarrage");
}

/// Marque demandant d'effacer les caches au prochain demarrage (ils sont en cours d'usage).
const PENDING: &str = "effacer-au-demarrage";

/// Demande d'effacer cookies et caches au prochain lancement.
pub fn wipe_next_start() {
    if let Err(error) = std::fs::write(crate::flags::data_dir().join(PENDING), b"") {
        warn!(%error, "marque d'effacement non ecrite");
    }
}

/// Parcourt les profils et conteneurs (deux niveaux : `profile/Default`, `profile/conteneur-x/Default`…).
fn wipe_traces(dir: &Path, depth: usize) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else { return 0 };
    let mut removed = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if TRACES.contains(&name.as_str()) {
            let gone = if path.is_dir() { std::fs::remove_dir_all(&path) } else { std::fs::remove_file(&path) };
            match gone {
                Ok(()) => removed += 1,
                Err(error) => warn!(%error, path = %path.display(), "trace non effacee"),
            }
        } else if path.is_dir() && depth < 3 {
            removed += wipe_traces(&path, depth + 1);
        }
    }
    removed
}
