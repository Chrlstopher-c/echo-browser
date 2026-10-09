//! Responsabilite : reprendre dans Echo les favoris et l'historique d'un autre navigateur (Chrome, Firefox, Brave…),
//! depuis l'accueil ou les Reglages. La lecture des bases se fait hors du thread interface.

use echo_contract::{CoreEvent, ImportSourceView, NoticeLevel};
use echo_library::import;

/// Dossier personnel ou chercher les autres navigateurs ; `ECHO_IMPORT_HOME` le remplace (bancs).
fn home() -> std::path::PathBuf {
    std::env::var_os("ECHO_IMPORT_HOME")
        .or_else(|| std::env::var_os("HOME"))
        .map(std::path::PathBuf::from)
        .unwrap_or_default()
}

/// Publie les navigateurs trouves.
pub fn publish_sources() {
    let sources = import::sources(&home())
        .into_iter()
        .map(|s| ImportSourceView { id: s.id, name: s.name })
        .collect();
    crate::bridge::publish(&CoreEvent::ImportSources { sources });
}

/// Reprend favoris et historique de la source `id`, puis previent avec le bilan.
pub fn run(id: &str) {
    let Some(source) = import::sources(&home()).into_iter().find(|s| s.id == id) else { return };
    let scratch = crate::flags::data_dir();
    std::thread::spawn(move || {
        let harvest = import::harvest(&source, &scratch);
        crate::containers::later(move || apply(&source.name, harvest));
    });
}

fn apply(name: &str, harvest: import::Harvest) {
    let counts = crate::session::with(|s| {
        let known: std::collections::HashSet<String> =
            echo_library::bookmarks::list(&s.library).into_iter().map(|b| b.url).collect();
        let mut added = 0;
        for (url, title) in harvest.bookmarks.iter().filter(|(url, _)| !known.contains(url)) {
            added += usize::from(echo_library::bookmarks::add(&s.library, url, title, None));
        }
        (added, echo_library::history::import_many(&s.library, &harvest.history))
    });
    let Some((bookmarks, pages)) = counts else { return };
    tracing::info!(name, bookmarks, pages, "import d'un autre navigateur");
    crate::bridge::library::publish_bookmarks();
    crate::bridge::publish_history("");
    crate::account::schedule::touch_soft();
    let message = format!("Importé de {name} : {bookmarks} favori(s), {pages} page(s) d’historique.");
    crate::bridge::publish(&CoreEvent::notice(NoticeLevel::Info, message));
}
