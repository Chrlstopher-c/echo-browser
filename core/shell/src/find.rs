//! Responsabilite : la recherche dans la page (Ctrl+F) — Chromium cherche et surligne, la barre affiche
//! « 3/12 » et mene les occurrences.

use cef::*;
use echo_contract::CoreEvent;

/// Cherche `text` dans l'onglet actif ; `next` passe a l'occurrence suivante (ou precedente si `!forward`).
pub fn find(text: &str, forward: bool, next: bool) {
    let Some(host) = active_host() else { return };
    if text.is_empty() {
        host.stop_finding(1);
        crate::bridge::publish(&CoreEvent::FindResult { count: 0, current: 0 });
        return;
    }
    tracing::debug!(%text, forward, next, "recherche dans la page");
    if !next {
        *PENDING_FIRST.lock() = Some(text.to_string());
    }
    host.find(Some(&CefString::from(text)), i32::from(forward), 0, i32::from(next));
}

/// Ferme la recherche : les surlignages disparaissent, la selection reste sur l'occurrence courante.
pub fn stop() {
    if let Some(host) = active_host() {
        host.stop_finding(0);
    }
}

/// Nouveau texte dont aucune occurrence n'est encore designee : quand la page n'a pas le focus (il est dans la barre),
/// Chromium compte sans activer la premiere ; on la designe alors une fois, comme le ferait Entree.
static PENDING_FIRST: parking_lot::Mutex<Option<String>> = parking_lot::Mutex::new(None);

/// Ferme la recherche de l'onglet actif en retirant aussi la selection (changement d'onglet).
pub fn stop_clearing() {
    if let Some(host) = active_host() {
        host.stop_finding(1);
    }
}

fn active_host() -> Option<BrowserHost> {
    crate::session::with(|s| s.tabs.active().and_then(|t| t.browser()).and_then(|b| b.host())).flatten()
}

wrap_find_handler! {
    pub struct EchoFind {
        marker: (),
    }

    impl FindHandler {
        fn on_find_result(
            &self,
            browser: Option<&mut Browser>,
            _identifier: i32,
            count: i32,
            _selection_rect: Option<&Rect>,
            active_match_ordinal: i32,
            final_update: i32,
        ) {
            if final_update == 1 && count > 0 && active_match_ordinal <= 0 {
                if let Some(text) = PENDING_FIRST.lock().take() {
                    if let Some(host) = browser.and_then(|b| b.host()) {
                        host.find(Some(&CefString::from(text.as_str())), 1, 0, 1);
                        return;
                    }
                }
            }
            if final_update == 1 && active_match_ordinal > 0 {
                PENDING_FIRST.lock().take();
            }
            crate::bridge::publish(&CoreEvent::FindResult { count, current: active_match_ordinal.max(0) });
        }
    }
}
