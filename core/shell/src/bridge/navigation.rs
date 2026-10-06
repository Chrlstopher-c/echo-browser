//! Responsabilite : la navigation demandee par l'interface ou le clavier — onglets
//! parcourus, adresses ouvertes, zoom, retour et avance.

use super::publish::publish_tabs;
use super::{close_tab, open_tab, publish, session};
use cef::{Browser, CefString, ImplBrowser, ImplBrowserHost, ImplFrame};
use echo_contract::CoreEvent;
use tracing::{debug, warn};

use crate::search;

/// Change le facteur de zoom d'un onglet.
pub(super) fn set_zoom(id: echo_contract::TabId, factor: f32) {
    let clamped = factor.clamp(0.25, 5.0);
    let host = session::with(|s| {
        if let Some(tab) = s.tabs.get_mut(id) {
            tab.zoom = clamped;
        }
        s.tabs.get_mut(id).and_then(|tab| tab.browser()).and_then(|b| b.host())
    })
    .flatten();
    if let Some(host) = host {
        // Chromium raisonne en niveaux, pas en facteurs : chaque niveau vaut 1,2 fois.
        host.set_zoom_level(f64::from(clamped).log(1.2));
    }
    publish_tabs();
}

/// Applique un raccourci clavier.
pub fn perform(action: crate::shortcuts::Action) {
    use crate::shortcuts::Action;
    match action {
        Action::NewTab => {
            open_tab(search::HOME);
            publish_tabs();
        }
        Action::CloseTab => {
            if let Some(id) = session::with(|s| s.tabs.active_id()).flatten() {
                close_tab(id);
            }
        }
        Action::NextTab => cycle_tab(1),
        Action::PreviousTab => cycle_tab(-1),
        Action::SelectTab(index) => {
            let target = session::with(|s| s.tabs.snapshot().get(index).map(|t| t.id)).flatten();
            if let Some(id) = target {
                super::select_tab(id);
            }
        }
        Action::Reload { bypass_cache } => with_browser(|browser| {
            if bypass_cache { browser.reload_ignore_cache() } else { browser.reload() }
        }),
        Action::FocusAddress => publish(&CoreEvent::FocusAddressRequested),
        Action::DismissOverlay => super::dismiss_overlays(),
        // F11 ne fait que sortir du plein ecran : c'est la page qui y entre, pas nous.
        Action::ToggleFullscreen => with_browser(|browser| {
            if let Some(host) = browser.host() {
                if host.is_fullscreen() == 1 {
                    host.exit_fullscreen(1);
                }
            }
        }),
    }
}

/// Passe a l'onglet suivant ou precedent, en bouclant.
pub(super) fn cycle_tab(step: isize) {
    let Some((ids, active)) = session::with(|s| {
        (s.tabs.snapshot().iter().map(|t| t.id).collect::<Vec<_>>(), s.tabs.active_id())
    }) else {
        return;
    };
    if ids.is_empty() {
        return;
    }
    let current = active.and_then(|id| ids.iter().position(|&x| x == id)).unwrap_or(0);
    let next = (current as isize + step).rem_euclid(ids.len() as isize) as usize;
    super::select_tab(ids[next]);
}

/// Recule ou avance dans l'onglet actif. Passe par Chromium quand il le peut, sinon
/// rejoue notre propre fil — c'est le cas apres une relance, ou son historique est neuf.
pub(super) fn travel(forward: bool) {
    let plan = session::with(|s| {
        let browser = s.tabs.active().and_then(|tab| tab.browser());
        let native = browser
            .map(|b| if forward { b.can_go_forward() == 1 } else { b.can_go_back() == 1 })
            .unwrap_or(false);
        if native {
            return Some(None);
        }
        let tab = s.tabs.active()?;
        let url = if forward { tab.next_url() } else { tab.previous_url() }?.to_string();
        Some(Some(url))
    })
    .flatten();

    match plan {
        Some(None) => with_browser(|browser| if forward { browser.go_forward() } else { browser.go_back() }),
        Some(Some(url)) => {
            session::with(|s| {
                if let Some(id) = s.tabs.active_id() {
                    if let Some(tab) = s.tabs.get_mut(id) {
                        tab.step(forward);
                    }
                }
            });
            debug!(%url, forward, "reprise du fil de navigation");
            navigate(&url);
        }
        None => debug!("aucun deplacement possible"),
    }
}

pub(super) fn with_browser(action: impl FnOnce(&Browser)) {
    let browser = session::with(|s| s.tabs.active().and_then(|tab| tab.browser())).flatten();
    match browser {
        Some(browser) => action(&browser),
        None => warn!("aucune vue de contenu : demande sans effet"),
    }
}

pub(super) fn navigate(url: &str) {
    let frame = session::with(|s| s.active_frame()).flatten();
    match frame {
        Some(frame) => {
            debug!(%url, "navigation");
            frame.load_url(Some(&CefString::from(url)));
        }
        None => warn!(%url, "navigation impossible : pas de frame de contenu"),
    }
}

/// Transforme ce que l'utilisateur tape en adresse : une URL telle quelle, sinon une recherche.
pub fn normalize(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.contains("://") || trimmed.starts_with("about:") {
        return trimmed.to_string();
    }
    let looks_like_host = !trimmed.contains(' ')
        && trimmed.split('/').next().is_some_and(|host| host.contains('.') && !host.ends_with('.'));
    if looks_like_host {
        return format!("https://{trimmed}");
    }
    search::query_url(trimmed)
}

pub(super) fn current_url() -> String {
    session::with(|s| s.active_url()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn une_adresse_reste_une_adresse() {
        assert_eq!(normalize("https://exemple.fr/page"), "https://exemple.fr/page");
        assert_eq!(normalize("exemple.fr"), "https://exemple.fr");
        assert_eq!(normalize("exemple.fr/page?a=1"), "https://exemple.fr/page?a=1");
    }

    #[test]
    fn des_mots_deviennent_une_recherche() {
        assert!(normalize("chat mignon").starts_with("https://www.google.com/search?q="));
        assert!(normalize("chat mignon").contains("chat+mignon"));
        assert!(normalize("rust cef").contains("rust+cef"));
    }

    #[test]
    fn un_mot_seul_sans_point_est_une_recherche() {
        assert!(normalize("meteo").starts_with("https://www.google.com/search?q="));
    }
}
