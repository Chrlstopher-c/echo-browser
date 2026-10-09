//! Responsabilite : les fiches de formulaire (identite, adresse) — lues dans le reglage synchronise `forms.cards`,
//! proposees au clic droit sur un champ, et versees dans les champs vides de la page. Jamais de mot de passe ni de
//! carte : la fiche n'en contient pas, et le script saute ces champs.

use cef::*;
use serde_json::Value;
use tracing::warn;

const FILL_JS: &str = include_str!("fill.js");
/// Fiches proposees au clic droit ; les suivantes restent dans les reglages.
pub const IN_MENU: usize = 3;

fn cards() -> Vec<Value> {
    let raw = crate::session::with(|s| {
        echo_library::settings::all(&s.library).into_iter().find(|(key, _)| key == "forms.cards").map(|(_, v)| v)
    })
    .flatten();
    let Some(echo_library::settings::Value::Text(text)) = raw else { return Vec::new() };
    serde_json::from_str::<Vec<Value>>(&text).unwrap_or_else(|err| {
        warn!(%err, "fiches de formulaire illisibles");
        Vec::new()
    })
}

/// Les noms des fiches proposees au clic droit.
pub fn names() -> Vec<String> {
    cards()
        .iter()
        .take(IN_MENU)
        .map(|card| card["name"].as_str().filter(|n| !n.trim().is_empty()).unwrap_or("Fiche").to_string())
        .collect()
}

/// Remplit le formulaire de la page active avec la fiche `index`, dans la trame qui a le focus (un formulaire peut
/// vivre dans un cadre).
pub fn fill(index: usize) {
    let Some(card) = cards().into_iter().nth(index) else { return };
    let Some(fields) = card.get("fields").filter(|f| f.is_object()) else { return };
    let browser = crate::session::with(|s| s.tabs.active().and_then(|t| t.browser())).flatten();
    let Some(frame) = browser.and_then(|b| b.focused_frame().or_else(|| b.main_frame())) else { return };
    let script = format!("({})({});", FILL_JS.trim().trim_end_matches(';'), fields);
    frame.execute_java_script(Some(&CefString::from(script.as_str())), Some(&CefString::from("echo://formulaires")), 0);
}

/// Script pose dans les pages quand l'utilisateur a des fiches : il signale l'entree dans un champ reconnu.
pub const OFFER_JS: &str = include_str!("offer.js");
/// Message console de ce script.
pub const FIELD_MARKER: &str = "echo:formulaire:champ";

/// Pages (par navigateur) ou la proposition a deja ete faite : une seule fois par page chargee.
static OFFERED: parking_lot::Mutex<Vec<i32>> = parking_lot::Mutex::new(Vec::new());

/// Faut-il poser le script ? Seulement si des fiches existent.
pub fn wanted() -> bool {
    !cards().is_empty()
}

/// Une nouvelle page : la proposition pourra revenir.
pub fn page_loaded(browser_id: i32) {
    OFFERED.lock().retain(|id| *id != browser_id);
}

/// L'utilisateur est entre dans un champ reconnu de l'onglet actif : la barre propose les fiches. Le remplissage ne
/// part que d'un clic dans la barre (requete `fillForm`), jamais de la page, qui ne peut que faire apparaitre l'offre.
pub fn offer(browser_id: i32) {
    let active = crate::session::with(|s| s.tabs.active().and_then(|t| t.browser()).map(|b| b.identifier())).flatten();
    if active != Some(browser_id) || OFFERED.lock().contains(&browser_id) {
        return;
    }
    let names = names();
    if names.is_empty() {
        return;
    }
    OFFERED.lock().push(browser_id);
    let actions = names
        .into_iter()
        .enumerate()
        .map(|(index, name)| echo_contract::NoticeAction {
            label: name,
            request: echo_contract::UiRequest::FillForm { index: index as u32 },
        })
        .collect();
    crate::bridge::publish(&echo_contract::CoreEvent::Notice {
        level: echo_contract::NoticeLevel::Info,
        message: "Remplir ce formulaire avec une fiche :".to_string(),
        actions,
    });
}
