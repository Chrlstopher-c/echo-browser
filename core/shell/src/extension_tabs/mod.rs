//! Onglets d'Echo vus par les extensions. Ils ne sont dans aucune fenetre Chrome : `chrome.tabs.query` ne rendait que
//! l'onglet vide de la fenetre d'ancrage (Proton Pass ne proposait rien pour le site ouvert). Echo charge un pont interne
//! (`pont/`, permission `debugger`) qui donne l'identifiant reel des onglets, et injecte dans les pages d'extension un
//! `tabs.query` qui s'en sert, avec l'ordre et l'onglet actif tenus par Echo.

use std::path::PathBuf;

use tracing::warn;

const PONT_ID: &str = "mcndjimfalplibhknmeieoolkckkpnnc";
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
