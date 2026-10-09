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
    let prefs = PREFS.lock().clone().unwrap_or_else(|| "null".to_string());
    let script = format!("(() => {{\n{READABILITY}\n({VIEW})(Readability, {prefs});\n}})();");
    frame.execute_java_script(Some(&CefString::from(script.as_str())), Some(&CefString::from("echo://lecture")), 0);
}

/// Reglages de lecture (taille, largeur, couleurs) retenus pour la session, en JSON deja verifie.
static PREFS: Mutex<Option<String>> = Mutex::new(None);

/// N'accepte que la forme attendue : une page ne peut rien glisser d'autre dans le script injecte.
fn checked_prefs(raw: &str) -> Option<String> {
    #[derive(serde::Deserialize, serde::Serialize)]
    struct Prefs {
        taille: u8,
        large: bool,
        ton: String,
    }
    let prefs: Prefs = serde_json::from_str(raw).ok()?;
    let ton_ok = ["auto", "clair", "sepia", "sombre"].contains(&prefs.ton.as_str());
    (ton_ok && (14..=28).contains(&prefs.taille)).then(|| serde_json::to_string(&prefs).ok()).flatten()
}

/// La vue de lecture est en place, ou ses reglages ont change.
pub fn shown(browser: &Browser, message: &str) {
    if let Some(raw) = message.strip_prefix("prefs:") {
        if reading(browser.identifier()) {
            if let Some(prefs) = checked_prefs(raw) {
                *PREFS.lock() = Some(prefs);
            }
        }
        return;
    }
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
