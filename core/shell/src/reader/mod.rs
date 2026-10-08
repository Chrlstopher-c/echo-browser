//! Responsabilite : le mode lecture — l'article de la page (extrait par Readability, celui de Firefox) affiche seul,
//! sans publicite ni menu, dans le theme d'Echo. On en sort en rechargeant la page. La vue est construite dans la page
//! meme : jamais dans une page interne d'Echo, que le contenu d'un site ne doit pas pouvoir toucher.

use std::collections::HashSet;

use cef::*;
use parking_lot::Mutex;

const READABILITY: &str = include_str!("readability.js");
const VIEW: &str = include_str!("lecture.js");
pub const MARKER: &str = "echo:lecture:";

/// Onglets (par navigateur) affiches en mode lecture.
static READING: Mutex<Option<HashSet<i32>>> = Mutex::new(None);

fn active_browser() -> Option<Browser> {
    crate::session::with(|s| s.tabs.active().and_then(|t| t.browser())).flatten()
}

fn reading(id: i32) -> bool {
    READING.lock().as_ref().is_some_and(|all| all.contains(&id))
}

/// L'onglet actif est-il en mode lecture ?
pub fn active_reading() -> bool {
    active_browser().is_some_and(|b| reading(b.identifier()))
}

/// Entre en mode lecture, ou en sort (la page d'origine revient).
pub fn toggle_active() {
    let Some(browser) = active_browser() else { return };
    if reading(browser.identifier()) {
        browser.reload();
        return;
    }
    let Some(frame) = browser.main_frame() else { return };
    if !CefString::from(&frame.url()).to_string().starts_with("http") {
        return;
    }
    let script = format!("(() => {{\n{READABILITY}\n({VIEW})(Readability);\n}})();");
    frame.execute_java_script(Some(&CefString::from(script.as_str())), Some(&CefString::from("echo://lecture")), 0);
}

/// La vue de lecture est en place.
pub fn shown(browser: &Browser, message: &str) {
    if message == "on" {
        READING.lock().get_or_insert_with(HashSet::new).insert(browser.identifier());
    }
}

/// Une nouvelle page s'est chargee : la vue de lecture n'y est plus.
pub fn loaded(browser: &Browser) {
    if let Some(all) = READING.lock().as_mut() {
        all.remove(&browser.identifier());
    }
}
