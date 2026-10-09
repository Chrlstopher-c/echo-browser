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

/// Marque demandant d'effacer les caches au prochain demarrage.
const PENDING: &str = "effacer-au-demarrage";

/// Efface maintenant, pour le profil affiche : l'historique depuis `since` (0 = tout), et/ou ses cookies (sessions de
/// sites) et son cache. Cookies et cache passent par l'onglet actif, qui porte le contexte du profil.
pub fn clear_now(since: i64, history: bool, cookies: bool, cache: bool) {
    use cef::{CefString, ImplBrowser, ImplBrowserHost};
    let forgotten = if history {
        crate::session::with(|s| echo_library::history::forget_since(&s.library, since, &s.tabs.space())).unwrap_or(0)
    } else {
        0
    };
    let host = crate::session::with(|s| s.tabs.active().and_then(|t| t.browser()).and_then(|b| b.host())).flatten();
    if let Some(host) = host {
        for (wanted, method) in [(cookies, "Network.clearBrowserCookies"), (cache, "Network.clearBrowserCache")] {
            if wanted {
                host.execute_dev_tools_method(0, Some(&CefString::from(method)), None);
            }
        }
    }
    info!(since, history, cookies, cache, forgotten, "donnees de navigation effacees");
    crate::bridge::publish_history("");
    crate::account::schedule::touch_soft();
    let message = "Données de navigation effacées.".to_string();
    crate::bridge::publish(&echo_contract::CoreEvent::notice(echo_contract::NoticeLevel::Info, message));
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
