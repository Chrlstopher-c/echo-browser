//! Onglets d'Echo vus par les extensions. Ils ne sont dans aucune fenetre Chrome : `chrome.tabs.query` ne rendait que
//! l'onglet vide de la fenetre d'ancrage (Proton Pass ne proposait rien pour le site ouvert). Echo charge un pont interne
//! (`pont/`, permission `debugger`) qui donne l'identifiant reel des onglets, et injecte dans les pages d'extension un
//! `tabs.query` qui s'en sert, avec l'ordre et l'onglet actif tenus par Echo.

mod workers;

use std::collections::HashSet;
use std::path::PathBuf;

use tracing::warn;

pub(crate) const PONT_ID: &str = "mcndjimfalplibhknmeieoolkckkpnnc";
const MANIFEST: &str = include_str!("pont/manifest.json");
const PONT_JS: &str = include_str!("pont/pont.js");
/// Fichiers du pont sans traitement : la regle des profils et la page qui l'applique a la demande.
const STATIC: [(&str, &str); 3] = [
    ("regle.js", include_str!("pont/regle.js")),
    ("appliquer.html", include_str!("pont/appliquer.html")),
    ("appliquer.js", include_str!("pont/appliquer.js")),
];
const POLYFILL: &str = include_str!("polyfill.js");

/// Le pont est interne : il ne s'affiche pas parmi les extensions de l'utilisateur.
pub fn is_pont(id: &str) -> bool {
    id == PONT_ID
}

/// Page du pont qui applique la regle des profils la ou elle est ouverte.
pub fn apply_page() -> String {
    format!("chrome-extension://{PONT_ID}/appliquer.html")
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
        // Chromium garde le script du service worker deja enregistre, meme quand la version change : le nom du
        // script suit son contenu, une nouvelle adresse force un nouvel enregistrement.
        let version = content_version();
        let script = format!("pont-{}.js", version.replace('.', "-"));
        let manifest = MANIFEST
            .replace("\"version\": \"1.0\"", &format!("\"version\": \"{version}\""))
            .replace("\"pont.js\"", &format!("\"{script}\""));
        for entry in std::fs::read_dir(&dir)?.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let stale = name.starts_with("pont") && name.ends_with(".js") && name != script;
            // `profils.json` : ancien registre lu par le pont, remplace par la marque `extensions`.
            if stale || name == "profils.json" {
                let _ = std::fs::remove_file(entry.path());
            }
        }
        let files = [("manifest.json", manifest.as_str()), (script.as_str(), PONT_JS)].into_iter().chain(STATIC);
        for (name, content) in files {
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

/// Version du pont tiree de son contenu (FNV-1a : stable d'une compilation a l'autre).
fn content_version() -> String {
    let hash = MANIFEST.bytes().chain(PONT_JS.bytes()).chain(STATIC.iter().flat_map(|(_, c)| c.bytes()))
        .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3));
    format!("1.{}.{}", (hash >> 16) % 65_535, hash % 65_535)
}

/// Acces a tous les sites : equivaut a voir les onglets.
const BROAD: [&str; 4] = ["<all_urls>", "*://*/*", "http://*/*", "https://*/*"];

/// Les extensions qui voient deja les onglets (permission `tabs` ou acces a tous les sites) : les seules a qui Echo
/// montre les siens.
fn allowed() -> HashSet<String> {
    crate::session::with(|s| {
        s.extensions
            .list()
            .into_iter()
            .filter(|e| e.permissions.iter().any(|p| p == "tabs" || BROAD.contains(&p.as_str())))
            .map(|e| e.id)
            .collect()
    })
    .unwrap_or_default()
}

/// Les onglets vivants, tels que les service workers des extensions les recoivent.
fn live_tabs() -> Vec<serde_json::Value> {
    crate::session::with(|s| {
        let active = s.tabs.active_id();
        s.tabs
            .iter()
            .filter(|t| !t.asleep)
            .map(|t| {
                serde_json::json!({"e": t.id, "url": t.url, "title": t.title, "pinned": t.pinned,
                    "status": if t.loading { "loading" } else { "complete" }, "active": Some(t.id) == active})
            })
            .collect()
    })
    .unwrap_or_default()
}

/// Demarre le service des onglets aux service workers des extensions (apres l'installation de la session).
pub fn start_workers() {
    workers::start(workers::Update { tabs: live_tabs(), allowed: Some(allowed()) });
}

/// Les onglets ont change : les service workers eveilles le sauront.
pub fn tabs_changed() {
    workers::send(workers::Update { tabs: live_tabs(), allowed: None });
}

/// Script a injecter dans une page d'extension, `None` pour le pont, les autres pages et les extensions qui ne voient
/// pas les onglets.
pub fn script_for(url: &str) -> Option<String> {
    let id = url.strip_prefix("chrome-extension://")?.split('/').next()?;
    if is_pont(id) || !allowed().contains(id) {
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
