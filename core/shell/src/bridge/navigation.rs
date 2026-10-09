//! Responsabilite : la navigation demandee par l'interface ou le clavier — onglets
//! parcourus, adresses ouvertes, zoom, retour et avance.

use super::publish::publish_tabs;
use super::{close_tab, open_tab, publish, session};
use cef::{Browser, CefString, ImplBrowser, ImplBrowserHost, ImplBrowserView, ImplFrame, ImplView};
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

/// Donne le focus clavier a la barre (la page le garde sinon : la saisie partirait dans la page), puis a l'adresse.
pub(super) fn focus_address() {
    focus_chrome();
    publish(&CoreEvent::FocusAddressRequested);
}

/// Montre la barre (meme repliee) et lui donne le clavier.
fn focus_chrome() {
    let chrome = session::with(|s| s.chrome.clone()).flatten();
    crate::window::reveal_chrome(true, chrome.as_ref());
    if let Some(view) = chrome {
        cef::View::from(&view).request_focus();
        if let Some(host) = view.browser().and_then(|b| b.host()) {
            host.set_focus(1);
        }
    }
}

/// Ctrl+F : la barre se montre, prend le clavier et ouvre la recherche dans la page.
pub(super) fn focus_find() {
    focus_chrome();
    publish(&CoreEvent::FindRequested);
}

/// Applique un raccourci clavier.
pub fn perform(action: crate::shortcuts::Action) {
    use crate::shortcuts::Action;
    match action {
        Action::NewPrivateTab => {
            if let Some(id) = super::open_tab_in(search::HOME, Some(crate::containers::PRIVATE)) {
                super::select_tab(id);
            }
            publish_tabs();
            focus_address();
        }
        Action::NewTab => {
            open_tab(search::HOME);
            publish_tabs();
            // Comme Chrome et Arc : un nouvel onglet, c'est d'abord une adresse ou une recherche a taper.
            focus_address();
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
        Action::FocusAddress => focus_address(),
        Action::Find => focus_find(),
        Action::ToggleSidebar => publish(&CoreEvent::ToggleSidebarRequested),
        Action::Reader => crate::reader::toggle_active(),
        Action::DismissOverlay => super::dismiss_overlays(),
        Action::ToggleDevTools => super::context::toggle_devtools(),
        // F11 ne fait que sortir du plein ecran : c'est la page qui y entre, pas nous.
        Action::ToggleFullscreen => with_browser(|browser| {
            if let Some(host) = browser.host() {
                if host.is_fullscreen() == 1 {
                    host.exit_fullscreen(1);
                }
            }
        }),
        other => perform_page(other),
    }
}

/// Raccourcis qui agissent sur la page active ou ouvrent quelque chose (Ctrl+O, Ctrl+D, zoom…).
fn perform_page(action: crate::shortcuts::Action) {
    use crate::shortcuts::{Action, ZoomStep};
    let active = session::with(|s| s.tabs.active().map(|t| (t.id, t.zoom, t.url.clone()))).flatten();
    match action {
        Action::OpenFile => crate::files::open_dialog(),
        Action::ReopenTab => reopen_closed(),
        Action::Back => travel(false),
        Action::Forward => travel(true),
        Action::Library => super::open_page_by_name("bibliotheque"),
        Action::Help => super::open_page_by_name("aide"),
        Action::ClearData => super::open_page_by_name("effacer"),
        Action::CopyUrl => {
            if let Some((_, _, url)) = active.filter(|(_, _, url)| !url.starts_with("echo://")) {
                super::context::copy(&url);
                publish(&CoreEvent::notice(echo_contract::NoticeLevel::Info, "Adresse copiée."));
            }
        }
        Action::Print => with_browser(|browser| {
            if let Some(host) = browser.host() {
                host.print();
            }
        }),
        Action::Bookmark => {
            if let Some((id, _, _)) = active {
                super::library::add_bookmark(id);
            }
        }
        Action::SavePage | Action::ViewSource => {
            if let Some((_, _, url)) = active.filter(|(_, _, url)| url.starts_with("http")) {
                save_or_source(action == Action::SavePage, &url);
            }
        }
        Action::Zoom(step) => {
            if let Some((id, zoom, _)) = active {
                set_zoom(id, match step {
                    ZoomStep::In => next_zoom(zoom, true),
                    ZoomStep::Out => next_zoom(zoom, false),
                    ZoomStep::Reset => 1.0,
                });
            }
        }
        _ => {}
    }
}

/// Paliers de zoom de Chrome et Firefox : 90, 100, 110, 125, 150 %… plutot que des multiples de 1,1.
const ZOOM_STEPS: [f32; 17] =
    [0.25, 0.33, 0.5, 0.67, 0.75, 0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0, 4.0, 5.0];

fn next_zoom(current: f32, up: bool) -> f32 {
    let found = if up {
        ZOOM_STEPS.iter().copied().find(|step| *step > current + 0.001)
    } else {
        ZOOM_STEPS.iter().rev().copied().find(|step| *step < current - 0.001)
    };
    found.unwrap_or(current)
}

/// Ctrl+S telecharge la page ; Ctrl+U ouvre son code source dans un onglet.
fn save_or_source(save: bool, url: &str) {
    if save {
        with_browser(|browser| {
            if let Some(host) = browser.host() {
                host.start_download(Some(&CefString::from(url)));
            }
        });
    } else {
        open_tab(&format!("view-source:{url}"));
        publish_tabs();
    }
}

/// Onglets fermes, le plus recent en dernier : (adresse, conteneur, place). Ctrl+Maj+T rouvre le dernier a sa place.
static CLOSED: parking_lot::Mutex<Vec<(String, Option<String>, Option<usize>)>> = parking_lot::Mutex::new(Vec::new());
const CLOSED_KEPT: usize = 25;

/// Retient un onglet qui va etre ferme (pages web seulement).
pub fn remember_closed(id: echo_contract::TabId) {
    let tab = session::with(|s| {
        let index = s.tabs.index_of(id);
        s.tabs.get_mut(id).map(|t| (t.url.clone(), t.container.clone(), index))
    })
    .flatten();
    let kept = tab.filter(|(url, container, _)| {
        (url.starts_with("http") || url.starts_with("file:")) && !crate::containers::is_private(container.as_deref())
    });
    if let Some((url, container, index)) = kept {
        let mut closed = CLOSED.lock();
        closed.push((url, container, index));
        if closed.len() > CLOSED_KEPT {
            closed.remove(0);
        }
    }
}

fn reopen_closed() {
    let last = CLOSED.lock().pop();
    if let Some((url, container, index)) = last {
        if let (Some(id), Some(index)) = (super::open_tab_in(&url, container.as_deref()), index) {
            session::with(|s| s.tabs.move_to(id, index));
        }
        publish_tabs();
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

/// Adresse demandee pendant qu'un onglet se reveille (sa page n'existe pas encore) : chargee des sa creation.
static PENDING: parking_lot::Mutex<Option<(echo_contract::TabId, String)>> = parking_lot::Mutex::new(None);

pub(super) fn navigate(url: &str) {
    let frame = session::with(|s| s.active_frame()).flatten();
    match frame {
        Some(frame) => {
            debug!(%url, "navigation");
            frame.load_url(Some(&CefString::from(url)));
        }
        None => match session::with(|s| s.tabs.active_id()).flatten() {
            Some(id) => {
                debug!(%url, id, "navigation gardee pour la fin du reveil");
                *PENDING.lock() = Some((id, url.to_string()));
            }
            None => warn!(%url, "navigation impossible : pas d'onglet actif"),
        },
    }
}

/// Une page vient d'etre creee : si une navigation l'attendait, elle part maintenant.
pub fn flush_pending(browser: &Browser) {
    let Some((id, _)) = PENDING.lock().clone() else { return };
    let owner = session::with(|s| s.tabs.by_browser(browser.identifier()).map(|t| t.id)).flatten();
    if owner != Some(id) {
        return;
    }
    if let (Some((_, url)), Some(frame)) = (PENDING.lock().take(), browser.main_frame()) {
        debug!(%url, "navigation apres reveil");
        frame.load_url(Some(&CefString::from(url.as_str())));
    }
}

/// Transforme ce que l'utilisateur tape en adresse : une URL telle quelle, sinon une recherche.
pub fn normalize(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.contains("://") || trimmed.starts_with("about:") || trimmed.starts_with("view-source:") {
        return trimmed.to_string();
    }
    if let Some(url) = crate::files::url_of_path(trimmed) {
        return url;
    }
    if is_local_server(trimmed) {
        return format!("http://{trimmed}");
    }
    let looks_like_host = !trimmed.contains(' ')
        && trimmed.split('/').next().is_some_and(|host| host.contains('.') && !host.ends_with('.'));
    if looks_like_host {
        return format!("https://{trimmed}");
    }
    search::query_url(trimmed)
}

/// `localhost`, `localhost:3000/…`, ou une adresse IP avec port : un serveur local, en http.
fn is_local_server(input: &str) -> bool {
    let host = input.split('/').next().unwrap_or_default();
    let (name, port) = host.split_once(':').map_or((host, None), |(n, p)| (n, Some(p)));
    let port_ok = port.is_none_or(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    let ip = name.split('.').count() == 4 && name.split('.').all(|part| part.parse::<u8>().is_ok());
    !input.contains(' ') && port_ok && (name == "localhost" || (ip && port.is_some()))
}

pub(super) fn current_url() -> String {
    session::with(|s| s.active_url()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn paliers_de_zoom() {
        assert_eq!(super::next_zoom(1.0, true), 1.1);
        assert_eq!(super::next_zoom(1.1, true), 1.25);
        assert_eq!(super::next_zoom(1.0, false), 0.9);
        assert_eq!(super::next_zoom(1.21, false), 1.1);
        assert_eq!(super::next_zoom(5.0, true), 5.0);
    }

    #[test]
    fn une_adresse_reste_une_adresse() {
        assert_eq!(normalize("https://exemple.fr/page"), "https://exemple.fr/page");
        assert_eq!(normalize("exemple.fr"), "https://exemple.fr");
        assert_eq!(normalize("exemple.fr/page?a=1"), "https://exemple.fr/page?a=1");
    }

    #[test]
    fn chemins_et_serveurs_locaux() {
        assert_eq!(normalize("/tmp/rapport.pdf"), "file:///tmp/rapport.pdf");
        assert!(normalize("~/Documents").starts_with("file:///"));
        assert_eq!(normalize("localhost:3000/app"), "http://localhost:3000/app");
        assert_eq!(normalize("192.168.1.10:8080"), "http://192.168.1.10:8080");
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
