//! Responsabilite : le canal entre l'interface et le coeur, dans les deux sens.
//!
//! L'interface poste ses demandes au schema interne ; elles sont mises en file et
//! rejouees sur le thread interface, seul endroit ou les objets Chromium sont manipulables.
//! Les evenements repartent par un appel de fonction dans la page.

pub mod library;
pub mod script;

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use echo_contract::{CoreEvent, ShieldView, TabId, UiRequest};
use parking_lot::Mutex;
use std::collections::VecDeque;
use tracing::{debug, info, warn};

use crate::session;

use crate::search;

/// File des demandes venues de l'interface. Remplie depuis le thread reseau, videe sur
/// le thread interface : seul du texte la traverse, jamais un objet Chromium.
static INBOX: Mutex<VecDeque<UiRequest>> = Mutex::new(VecDeque::new());

/// Enregistre une demande et programme son traitement. Appelable depuis n'importe quel thread.
pub fn submit(raw: &[u8]) {
    let request: UiRequest = match serde_json::from_slice(raw) {
        Ok(request) => request,
        Err(err) => {
            warn!(%err, "demande de l'interface illisible, ignoree");
            return;
        }
    };
    INBOX.lock().push_back(request);
    let mut task = DrainTask::new(());
    post_task(ThreadId::UI, Some(&mut task));
}

wrap_task! {
    struct DrainTask {
        marker: (),
    }

    impl Task {
        fn execute(&self) {
            drain();
        }
    }
}

/// Traite toutes les demandes en attente. S'execute sur le thread interface.
fn drain() {
    loop {
        let Some(request) = INBOX.lock().pop_front() else { return };
        debug!(?request, "demande de l'interface");
        apply(request);
    }
}

fn apply(request: UiRequest) {
    match request {
        UiRequest::Navigate { input, .. } => navigate(&normalize(&input)),
        UiRequest::GoBack { .. } => travel(false),
        UiRequest::GoForward { .. } => travel(true),
        UiRequest::Reload { bypass_cache, .. } => with_browser(|browser| {
            if bypass_cache { browser.reload_ignore_cache() } else { browser.reload() }
        }),
        UiRequest::Stop { .. } => with_browser(|browser| browser.stop_load()),
        UiRequest::RefreshFilterLists { force } => refresh_lists(force),
        UiRequest::SetFilterListEnabled { id, enabled } => {
            let known = session::with(|s| s.shield.set_list_enabled(&id, enabled)).unwrap_or(false);
            if !known {
                notify_error(&format!("liste de filtres inconnue : {id}"));
            }
            publish_filter_lists();
            publish_shield();
        }
        UiRequest::SetShieldEnabled { enabled } => {
            session::with(|s| s.shield.set_enabled(enabled));
            publish_shield();
        }
        UiRequest::ToggleShieldForSite { .. } => {
            let url = current_url();
            session::with(|s| s.shield.toggle_site(&url));
            publish_shield();
        }
        UiRequest::NewTab { url } => {
            let target = url.map(|u| normalize(&u)).unwrap_or_else(|| search::HOME.to_string());
            open_tab(&target);
            publish_tabs();
        }
        UiRequest::SelectTab { id } => {
            dismiss_overlays();
            session::with(|s| s.tabs.select(id));
            publish_tabs();
            publish_shield();
        }
        UiRequest::CloseTab { id } => close_tab(id),
        UiRequest::SetChromeWidth { pixels } => {
            let chrome = session::with(|s| s.chrome.clone()).flatten();
            crate::window::set_chrome_width(pixels as i32, chrome.as_ref());
        }
        UiRequest::SetAccent { color } => {
            let host = session::with(|s| s.tabs.host()).flatten();
            crate::window::set_accent(&color, host.as_ref());
        }
        UiRequest::InstallExtension { source } => open_store(&source),
        UiRequest::OpenExtensionPopup { id, anchor } => open_extension_popup(&id, anchor),
        UiRequest::CloseExtensionPopup => {
            crate::overlay::close_extension_popup();
            publish(&CoreEvent::ExtensionPopupChanged { id: None });
        }
        UiRequest::RemoveExtension { id } => {
            let ours = session::with(|s| {
                s.extensions.list().into_iter().find(|e| e.id == id).map(|e| e.from_command_line)
            })
            .flatten()
            .unwrap_or(false);
            if ours {
                let outcome = session::with(|s| s.extensions.remove(&id));
                if let Some(Err(err)) = outcome {
                    notify_error(&format!("suppression impossible : {err}"));
                } else {
                    mark_restart_needed();
                }
                publish_extensions();
            } else {
                // Celles du catalogue appartiennent au gestionnaire de Chromium :
                // les effacer dans son dos laisserait son profil incoherent.
                open_tab(echo_extensions::profile::MANAGE_PAGE);
                publish_tabs();
            }
        }
        UiRequest::SetExtensionEnabled { id, enabled } => {
            let outcome = session::with(|s| s.extensions.set_enabled(&id, enabled));
            if let Some(Err(err)) = outcome {
                notify_error(&format!("changement impossible : {err}"));
            } else {
                mark_restart_needed();
            }
            publish_extensions();
        }
        UiRequest::ExitFullscreen => with_browser(|browser| {
            if let Some(host) = browser.host() {
                host.exit_fullscreen(1);
            }
        }),
        UiRequest::RestartBrowser => restart_browser(),

        UiRequest::AddBookmark { id } => library::add_bookmark(id),
        UiRequest::RemoveBookmark { url } => library::remove_bookmark(&url),
        UiRequest::MoveBookmark { url, to } => library::move_bookmark(&url, to),
        UiRequest::RemoveHistoryEntry { url, visited_at } => {
            library::remove_history_entry(&url, visited_at)
        }
        UiRequest::ClearHistory => library::clear_history(),
        UiRequest::SearchHistory { terms } => library::publish_history(&terms),
        UiRequest::ForgetDownload { id } => library::forget_download(id),
        UiRequest::CancelDownload { id } => {
            crate::transfers::request_cancel(id);
        }
        UiRequest::OpenDownload { id } => library::open_download(id, false),
        UiRequest::RevealDownload { id } => library::open_download(id, true),
        UiRequest::UpdateSetting { key, value } => library::update_setting(&key, &value),
        UiRequest::OpenExtensionManager => {
            open_tab(echo_extensions::profile::MANAGE_PAGE);
            publish_tabs();
        }
        UiRequest::PinTab { id, pinned } => {
            session::with(|s| {
                if let Some(tab) = s.tabs.get_mut(id) {
                    tab.pinned = pinned;
                }
            });
            publish_tabs();
        }
        UiRequest::SetZoom { id, factor } => set_zoom(id, factor),
        UiRequest::MoveTab { id, to } => {
            session::with(|s| s.tabs.move_to(id, to));
            publish_tabs();
        }
        other => debug!(?other, "demande pas encore traitee"),
    }
}

/// Ouvre un onglet. Chaque appel a Chromium se fait hors de l'acces a l'etat : la creation
/// de la vue et son rattachement declenchent des rappels qui veulent lire cet etat.
/// Referme ce qui est pose au-dessus de la page. A appeler des que le contenu change :
/// une fenetre d'extension qui survit a un changement d'onglet flotte dans le vide.
fn dismiss_overlays() {
    if crate::overlay::open_popup_id().is_some() {
        crate::overlay::close_extension_popup();
        publish(&CoreEvent::ExtensionPopupChanged { id: None });
    }
}

pub fn open_tab(url: &str) {
    dismiss_overlays();
    let Some((mut client, host)) = session::with(|s| (s.client.clone(), s.tabs.host())) else {
        return;
    };
    let Some(view) = crate::window::create_view(client.as_mut(), url, 0) else {
        warn!(%url, "vue d'onglet non creee");
        return;
    };
    if let Some(host) = host {
        let mut child = View::from(&view);
        host.add_child_view(Some(&mut child));
    }
    session::with(|s| s.tabs.adopt(view, url));
}

/// Ferme un onglet. Comme pour l'ouverture, les appels a Chromium se font hors de
/// l'acces a l'etat, sinon la fermeture fige le navigateur.
pub fn close_tab(id: TabId) {
    let Some(detached) = session::with(|s| s.tabs.detach(id)) else { return };
    let remaining = detached.remaining;
    detached.dispose();
    if remaining == 0 {
        quit_message_loop();
        return;
    }
    session::with(|s| s.tabs.refresh_visibility());
    publish_tabs();
    publish_shield();
}

/// Ouvre la fiche d'une extension dans un onglet, pour que Chromium mene l'installation.
///
/// Chromium sait installer depuis le catalogue, avec sa demande de permissions et sa
/// prise en compte immediate. Telecharger le paquet nous-memes ferait un second
/// inventaire, invisible de son gestionnaire, et imposerait une relance.
fn open_store(source: &str) {
    let target = echo_extensions::catalog::extract_id(source)
        .map(|id| echo_extensions::Extensions::store_page(&id))
        .unwrap_or_else(|| CATALOG_HOME.to_string());
    info!(%target, "ouverture du catalogue");
    open_tab(&target);
    publish_tabs();
}

/// Page d'accueil du catalogue.
const CATALOG_HOME: &str = "https://chromewebstore.google.com/";

/// Vrai quand une relance est necessaire pour que les extensions prennent effet.
static RESTART_NEEDED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn mark_restart_needed() {
    RESTART_NEEDED.store(true, std::sync::atomic::Ordering::SeqCst);
}

fn notify_error(message: &str) {
    warn!(%message, "demande refusee");
    publish(&CoreEvent::Notice {
        level: echo_contract::NoticeLevel::Error,
        message: message.to_string(),
    });
}

/// Diffuse l'inventaire des extensions.
pub fn publish_extensions() {
    let pending = RESTART_NEEDED.load(std::sync::atomic::Ordering::SeqCst);
    let Some(extensions) = session::with(|s| s.extensions.list()) else { return };
    let view = extensions
        .into_iter()
        .map(|extension| {
            let url = |path: &str| echo_extensions::action::resource_url(&extension.id, path);
            echo_contract::ExtensionView {
                name: extension.name.clone(),
                version: extension.version.clone(),
                enabled: extension.enabled,
                // Seules celles que nous chargeons nous-memes attendent une relance ;
                // celles du catalogue sont prises en compte immediatement par Chromium.
                pending: pending && extension.from_command_line,
                removable: extension.from_command_line,
                // L'icone passe par notre schema : celui de l'extension ne se lit pas
                // depuis une page interne (voir assets::ICON_HOST).
                icon: extension
                    .action
                    .icon
                    .as_ref()
                    .map(|_| format!("echo://{}/{}", crate::assets::ICON_HOST, extension.id)),
                popup: extension.action.popup.as_deref().map(&url),
                id: extension.id.clone(),
            }
        })
        .collect();
    publish(&CoreEvent::ExtensionsChanged { extensions: view, restart_pending: pending });
}

/// Enregistre les onglets, previent l'interface, puis relance apres un court delai
/// pour lui laisser le temps d'afficher son ecran d'attente.
pub fn restart_browser() {
    let snapshot = session::with(|s| s.tabs.to_snapshot());
    if let Some(snapshot) = snapshot {
        crate::restart::save(&crate::flags::data_dir(), &snapshot);
    }
    publish(&CoreEvent::Restarting {
        reason: "Application des extensions".to_string(),
    });
    crate::restart::request();

    let mut task = QuitTask::new(());
    post_delayed_task(ThreadId::UI, Some(&mut task), 400);
}

wrap_task! {
    struct QuitTask {
        marker: (),
    }

    impl Task {
        fn execute(&self) {
            quit_message_loop();
        }
    }
}

/// Change le facteur de zoom d'un onglet.
fn set_zoom(id: echo_contract::TabId, factor: f32) {
    let clamped = factor.clamp(0.25, 5.0);
    let host = session::with(|s| {
        if let Some(tab) = s.tabs.get_mut(id) {
            tab.zoom = clamped;
        }
        s.tabs.get_mut(id).and_then(|tab| tab.view.browser()).and_then(|b| b.host())
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
                session::with(|s| s.tabs.select(id));
                publish_tabs();
            }
        }
        Action::Reload { bypass_cache } => with_browser(|browser| {
            if bypass_cache { browser.reload_ignore_cache() } else { browser.reload() }
        }),
        Action::FocusAddress => publish(&CoreEvent::FocusAddressRequested),
        Action::DismissOverlay => {
            crate::overlay::close_extension_popup();
            publish(&CoreEvent::ExtensionPopupChanged { id: None });
        }
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
fn cycle_tab(step: isize) {
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
    session::with(|s| s.tabs.select(ids[next]));
    publish_tabs();
    publish_shield();
}

/// Recule ou avance dans l'onglet actif. Passe par Chromium quand il le peut, sinon
/// rejoue notre propre fil — c'est le cas apres une relance, ou son historique est neuf.
fn travel(forward: bool) {
    let plan = session::with(|s| {
        let browser = s.tabs.active().and_then(|tab| tab.view.browser());
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

fn with_browser(action: impl FnOnce(&Browser)) {
    let browser = session::with(|s| s.tabs.active().and_then(|tab| tab.view.browser())).flatten();
    match browser {
        Some(browser) => action(&browser),
        None => warn!("aucune vue de contenu : demande sans effet"),
    }
}

fn navigate(url: &str) {
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

fn current_url() -> String {
    session::with(|s| s.active_url()).unwrap_or_default()
}

/// Pousse un evenement vers l'interface.
pub fn publish(event: &CoreEvent) {
    let payload = match serde_json::to_string(event) {
        Ok(payload) => payload,
        Err(err) => {
            warn!(%err, "evenement non serialisable");
            return;
        }
    };
    let script = format!("window.__echoDeliver && window.__echoDeliver({payload})");
    let frame = session::with(|s| s.chrome_frame()).flatten();
    match frame {
        Some(frame) => frame.execute_java_script(Some(&CefString::from(script.as_str())), None, 0),
        None => debug!("interface pas encore prete, evenement perdu"),
    }
}

/// Envoie a l'interface tout ce qu'elle doit savoir pour s'afficher.
pub fn publish_initial_state() {
    publish_tabs();
    publish_shield();
    publish_extensions();
    library::publish_bookmarks();
    library::publish_history("");
    library::publish_downloads();
    library::publish_settings();
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
fn refresh_lists(force: bool) {
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
    let Some(state) = session::with(|s| {
        let tally = s.shield.tally(0);
        ShieldView {
            enabled: s.shield.is_enabled(),
            active_here: s.shield.is_active_for(&url),
            blocked_here: tally.tab,
            blocked_total: tally.total,
        }
    }) else {
        return;
    };
    publish(&CoreEvent::ShieldUpdated { id: 0, state });
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

/// Ouvre la fenetre d'une extension sous son icone, ou la referme si c'est deja elle.
///
/// L'adresse de la fenetre vient du manifeste, jamais de l'interface : une adresse
/// choisie par la page afficherait n'importe quoi au-dessus du contenu.
fn open_extension_popup(id: &str, anchor: echo_contract::AnchorRect) {
    let found = session::with(|s| {
        s.extensions.list().into_iter().find(|extension| extension.id == id)
    })
    .flatten();
    let Some(extension) = found else {
        tracing::warn!(%id, "fenetre d'extension : extension inconnue");
        return;
    };
    let Some(path) = extension.action.popup.as_deref() else {
        tracing::warn!(%id, "cette extension ne declare pas de fenetre");
        return;
    };
    let url = echo_extensions::action::resource_url(id, path);

    // L'appel a Chromium se fait hors de tout acces a l'etat : il rappelle le programme
    // pendant la creation de la vue.
    let Some(chrome) = session::with(|s| s.chrome.clone()).flatten() else { return };
    let rect = cef::Rect { x: anchor.x, y: anchor.y, width: anchor.width, height: anchor.height };
    crate::overlay::toggle_extension_popup(id, &url, rect, &chrome);
    publish(&CoreEvent::ExtensionPopupChanged { id: crate::overlay::open_popup_id() });
}
