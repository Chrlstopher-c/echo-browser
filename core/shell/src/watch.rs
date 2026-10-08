//! Responsabilite : pages surveillees — a chaque visite d'une page surveillee, son texte est lu et compare a celui de la
//! visite precedente ; s'il a change, l'interface montre les lignes ajoutees et retirees. Seul le texte lu a notre
//! demande est cru (une page qui imite le message de console hors de cette fenetre est ignoree).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use cef::*;
use echo_contract::CoreEvent;
use echo_library::watched;
use parking_lot::Mutex;

pub const MARKER: &str = "echo:texte:";
const READ_SCRIPT: &str = "setTimeout(()=>console.debug('echo:texte:'+JSON.stringify(\
(document.body?document.body.innerText:'').slice(0,60000))),1500)";
const ARMED_FOR: Duration = Duration::from_secs(15);

static ARMED: Mutex<Option<HashMap<i32, Instant>>> = Mutex::new(None);

fn read(browser: &Browser) {
    ARMED.lock().get_or_insert_with(HashMap::new).insert(browser.identifier(), Instant::now());
    if let Some(frame) = browser.main_frame() {
        frame.execute_java_script(Some(&CefString::from(READ_SCRIPT)), Some(&CefString::from("echo://surveille")), 0);
    }
}

fn page_of(browser: &Browser) -> String {
    browser.main_frame().map(|f| CefString::from(&f.url()).to_string()).unwrap_or_default()
}

/// Fin de chargement d'une page principale : si elle est surveillee, on lit son texte.
pub fn loaded(browser: &Browser, url: &str) {
    if crate::session::with(|s| watched::is_watched(&s.library, url)).unwrap_or(false) {
        read(browser);
    }
}

/// Le texte demande est arrive.
pub fn text(browser: &Browser, message: &str) {
    let armed = ARMED.lock().as_mut().and_then(|all| all.remove(&browser.identifier()));
    if !armed.is_some_and(|at| at.elapsed() < ARMED_FOR) {
        return;
    }
    let Ok(text) = serde_json::from_str::<String>(message) else { return };
    let url = page_of(browser);
    let change = crate::session::with(|s| watched::visited(&s.library, &url, &text)).flatten();
    if let Some(change) = change {
        crate::bridge::publish(&CoreEvent::PageChanged { url, added: change.added, removed: change.removed });
    }
}

fn active_browser() -> Option<Browser> {
    crate::session::with(|s| s.tabs.active().and_then(|t| t.browser())).flatten()
}

pub fn active_watched() -> bool {
    let Some(browser) = active_browser() else { return false };
    let url = page_of(&browser);
    crate::session::with(|s| watched::is_watched(&s.library, &url)).unwrap_or(false)
}

/// Surveiller la page active : son texte actuel devient la reference.
pub fn watch_active() {
    let Some(browser) = active_browser() else { return };
    let url = page_of(&browser);
    if url.starts_with("http") {
        crate::session::with(|s| watched::watch(&s.library, &url));
        read(&browser);
    }
}

pub fn unwatch_active() {
    let Some(browser) = active_browser() else { return };
    let url = page_of(&browser);
    crate::session::with(|s| watched::unwatch(&s.library, &url));
}
