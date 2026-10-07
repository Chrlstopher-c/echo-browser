//! Onglets d'Echo vus par les extensions. Ils ne sont dans aucune fenetre Chrome : `chrome.tabs.query` ne rendait que
//! l'onglet vide de la fenetre d'ancrage (Proton Pass ne proposait rien pour le site ouvert). Echo charge un pont interne
//! (`pont/`, permission `debugger`) qui donne l'identifiant reel des onglets, et injecte dans les pages d'extension un
//! `tabs.query` qui s'en sert, avec l'ordre et l'onglet actif tenus par Echo.

use std::path::PathBuf;

use tracing::warn;

const PONT_ID: &str = "mcndjimfalplibhknmeieoolkckkpnnc";
const MANIFEST: &str = include_str!("pont/manifest.json");
const PONT_JS: &str = include_str!("pont/pont.js");
const POLYFILL: &str = include_str!("polyfill.js");

/// Le pont est interne : il ne s'affiche pas parmi les extensions de l'utilisateur.
pub fn is_pont(id: &str) -> bool {
    id == PONT_ID
}

/// Dossier du pont, a charger par `--load-extension`.
pub fn pont_dir() -> PathBuf {
    crate::flags::data_dir().join("echo-pont")
}

/// Ecrit le pont livre avec cette version dans le repertoire de travail. `None` s'il n'a pas pu l'etre.
pub fn install_pont() -> Option<PathBuf> {
    let dir = pont_dir();
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(&dir)?;
        for (name, content) in [("manifest.json", MANIFEST), ("pont.js", PONT_JS)] {
            let path = dir.join(name);
            if std::fs::read_to_string(&path).ok().as_deref() != Some(content) {
                std::fs::write(path, content)?;
            }
        }
        Ok(())
    };
    match write() {
        Ok(()) => Some(dir),
        Err(err) => {
            warn!(%err, "pont des extensions non installe : tabs.query ne verra pas les onglets d'Echo");
            None
        }
    }
}

/// Script a injecter dans une page d'extension, `None` pour le pont lui-meme et les autres pages.
pub fn script_for(url: &str) -> Option<String> {
    let id = url.strip_prefix("chrome-extension://")?.split('/').next()?;
    if is_pont(id) {
        return None;
    }
    let tabs = crate::session::with(|s| {
        let active = s.tabs.active_id();
        let space = s.tabs.space();
        s.tabs
            .iter()
            .filter(|t| t.space == space && !t.asleep)
            .map(|t| serde_json::json!({"url": t.url, "title": t.title, "pinned": t.pinned, "active": Some(t.id) == active}))
            .collect::<Vec<_>>()
    })?;
    let data = serde_json::json!({ "tabs": tabs }).to_string();
    Some(POLYFILL.replace("__DATA__", &data).replace("__PONT__", PONT_ID))
}
