//! Responsabilite : ce que le coeur pousse vers l'interface — etat de depart, onglets,
//! bouclier, listes de filtres, plein ecran.

use super::navigation::current_url;
use super::{extensions, library, publish, session};
// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee.
use cef::*;
use echo_contract::{CoreEvent, ShieldView};
use tracing::{info, warn};

/// Envoie a l'interface tout ce qu'elle doit savoir pour s'afficher.
pub fn publish_initial_state() {
    publish_tabs();
    publish_shield();
    extensions::publish_extensions();
    library::publish_bookmarks();
    library::publish_history("");
    library::publish_downloads();
    library::publish_settings();
    library::publish_permissions();
    crate::codecs::publish();
    publish_filter_lists();
}

/// Diffuse l'etat des listes de filtres.
pub fn publish_filter_lists() {
    let Some((subs, rules, refreshed_at)) = session::with(|s| {
        (s.shield.subscriptions(), s.shield.rules_per_list(), s.shield.refreshed_at())
    }) else {
        return;
    };
    let lists = subs
        .into_iter()
        .map(|sub| echo_contract::FilterListView {
            rules: rules.iter().find(|(id, _)| *id == sub.id).and_then(|(_, count)| *count),
            id: sub.id,
            title: sub.title,
            enabled: sub.enabled,
        })
        .collect();
    publish(&CoreEvent::FilterListsChanged { lists, refreshed_at });
}

/// Rafraichit les listes hors du thread interface : le telechargement est long.
pub(super) fn refresh_lists(force: bool) {
    let Some(shield) = session::with(|s| s.shield.clone()) else { return };
    std::thread::spawn(move || {
        match shield.refresh_lists(force) {
            Ok(count) => info!(listes = count, "listes rafraichies"),
            Err(err) => warn!(%err, "rafraichissement incomplet"),
        }
        let mut task = RefreshDoneTask::new(());
        post_task(ThreadId::UI, Some(&mut task));
    });
}

wrap_task! {
    struct RefreshDoneTask {
        marker: (),
    }

    impl Task {
        fn execute(&self) {
            publish_filter_lists();
            publish_shield();
        }
    }
}

/// Diffuse l'etat courant du bouclier.
pub fn publish_shield() {
    let url = current_url();
    // L'interface lit l'etat du bouclier sous l'identifiant de l'onglet actif : l'envoyer sous un autre
    // (l'ancien 0 fixe) lui faisait afficher « actif » par defaut, quel que soit l'etat reel.
    let Some((id, state)) = session::with(|s| {
        let tally = s.shield.tally(0);
        let view = ShieldView {
            enabled: s.shield.is_enabled(),
            active_here: s.shield.is_active_for(&url),
            blocked_here: tally.tab,
            blocked_total: tally.total,
        };
        (s.tabs.active_id().unwrap_or(0), view)
    }) else {
        return;
    };
    publish(&CoreEvent::ShieldUpdated { id, state });
}

/// Met a jour un onglet a partir de ce que Chromium rapporte, puis previent l'interface.
pub fn publish_tab(browser_id: i32, url: &str, title: &str, loading: bool) {
    let known = session::with(|s| {
        let tab = s.tabs.by_browser(browser_id)?;
        tab.url = url.to_string();
        tab.title = if title.is_empty() { url.to_string() } else { title.to_string() };
        tab.loading = loading;
        if !loading {
            tab.record_visit(url);
        }
        Some(tab.title.clone())
    })
    .flatten();

    let Some(title) = known else { return };
    if !loading {
        library::record_visit(url, &title);
    }
    publish_tabs();
}

/// Retient l'icone du site pour l'onglet, et previent l'interface.
pub fn set_tab_favicon(browser_id: i32, icon: &str) {
    let changed = session::with(|s| {
        let tab = s.tabs.by_browser(browser_id)?;
        let new = Some(icon.to_string());
        let changed = tab.favicon != new;
        tab.favicon = new;
        Some(changed)
    })
    .flatten()
    .unwrap_or(false);
    if changed {
        publish_tabs();
    }
}

/// Note qu'une page porte une saisie de l'utilisateur, ou qu'elle repart de zero.
pub fn set_tab_scroll(browser_id: i32, scroll: i32) {
    session::with(|s| {
        if let Some(tab) = s.tabs.by_browser(browser_id) {
            tab.scroll = scroll.max(0);
        }
    });
}

/// Une nouvelle page commence : son defilement repart de zero, sauf si on rend celui d'un reveil.
pub fn reset_tab_scroll(browser_id: i32) {
    session::with(|s| {
        if let Some(tab) = s.tabs.by_browser(browser_id) {
            if tab.pending_scroll.is_none() {
                tab.scroll = 0;
            }
        }
    });
}

/// Le defilement a rejouer pour cet onglet reveille, une seule fois.
pub fn take_pending_scroll(browser_id: i32) -> Option<i32> {
    session::with(|s| s.tabs.by_browser(browser_id).and_then(|tab| tab.pending_scroll.take())).flatten()
}

/// La page signale une lecture (video ou son) : l'onglet ne doit pas dormir, la marque « son » suit.
pub fn set_tab_media(browser_id: i32, playing: bool, audible: bool) {
    let changed = session::with(|s| {
        let tab = s.tabs.by_browser(browser_id)?;
        let changed = tab.audible != audible;
        tab.playing = playing;
        tab.audible = audible;
        Some(changed)
    })
    .flatten()
    .unwrap_or(false);
    if changed {
        publish_tabs();
    }
}

pub fn set_tab_dirty(browser_id: i32, dirty: bool) {
    session::with(|s| {
        if let Some(tab) = s.tabs.by_browser(browser_id) {
            tab.dirty = dirty;
        }
    });
}

/// Note le titre rendu par la page, et le repercute dans l'historique.
pub fn set_tab_title(browser_id: i32, title: &str) {
    if title.is_empty() {
        return;
    }
    let url = session::with(|s| {
        let tab = s.tabs.by_browser(browser_id)?;
        tab.title = title.to_string();
        Some(tab.url.clone())
    })
    .flatten();
    if let Some(url) = url {
        library::record_visit(&url, title);
        publish_tabs();
    }
}

/// Signale a l'interface que la page occupe tout l'ecran, ou n'en occupe plus.
pub fn set_fullscreen(active: bool) {
    info!(active, "plein ecran");
    publish(&CoreEvent::FullscreenChanged { active });
    let chrome = session::with(|s| s.chrome.clone()).flatten();
    // La bande laterale se replie a zero : sans cela elle resterait posee sur la video.
    let width = if active { 0 } else { crate::window::CHROME_WIDTH };
    crate::window::set_chrome_width(width, chrome.as_ref());
}

pub fn publish_tabs() {
    let Some((tabs, active)) = session::with(|s| (s.tabs.snapshot(), s.tabs.active_id())) else {
        return;
    };
    publish(&CoreEvent::TabsChanged { tabs, active });
    crate::persist::schedule();
    crate::extension_tabs::tabs_changed();
}
