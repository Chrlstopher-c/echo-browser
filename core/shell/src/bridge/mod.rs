//! Responsabilite : le canal entre l'interface et le coeur, dans les deux sens.
//!
//! L'interface poste ses demandes au schema interne ; elles sont mises en file et
//! rejouees sur le thread interface, seul endroit ou les objets Chromium sont manipulables.
//! Les evenements repartent par un appel de fonction dans la page.

pub mod context;
mod lifecycle;
pub use lifecycle::*;
pub mod extensions;
pub mod navigation;
pub mod library;
pub mod publish;

pub use extensions::publish_extensions;
pub use library::{publish_bookmarks, publish_history, publish_permissions, publish_settings};
pub use publish::{
    reset_tab_scroll, set_tab_dirty, set_tab_media, set_tab_favicon, set_tab_scroll, take_pending_scroll,
    publish_filter_lists, publish_initial_state, publish_shield, publish_tab, publish_tabs,
    set_fullscreen, set_tab_title,
};
pub use navigation::{flush_pending, http_fallback, normalize, perform};
use navigation::{current_url, navigate, set_zoom, travel, with_browser};
use publish::refresh_lists;
pub mod script;

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use echo_contract::{CoreEvent, UiRequest};
use parking_lot::Mutex;
use std::collections::VecDeque;
use tracing::{debug, warn};

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
        let tabs_change =
            matches!(request, UiRequest::NewTab { .. } | UiRequest::CloseTab { .. } | UiRequest::PinTab { .. });
        let touches_synced = changes_synced_data(&request);
        apply(request);
        if touches_synced {
            crate::account::schedule::touch();
        } else if tabs_change {
            crate::account::schedule::touch_soft();
        }
    }
}

/// Demandes qui modifient une donnee synchronisee par le compte (reglages, favoris, historique, extensions, onglets).
fn changes_synced_data(request: &UiRequest) -> bool {
    matches!(
        request,
        UiRequest::UpdateSetting { .. }
            | UiRequest::AddBookmark { .. }
            | UiRequest::RemoveBookmark { .. }
            | UiRequest::MoveBookmark { .. }
            | UiRequest::RemoveHistoryEntry { .. }
            | UiRequest::ClearHistory
            | UiRequest::InstallExtension { .. }
            | UiRequest::RemoveExtension { .. }
            | UiRequest::SetShieldEnabled { .. }
    )
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
            session::with(|s| {
                s.shield.set_enabled(enabled);
                let _ = echo_library::settings::set(&s.library, "shield.enabled", &echo_library::settings::Value::Flag(enabled));
            });
            publish_shield();
            reload_active();
        }
        UiRequest::ToggleShieldForSite { .. } => {
            let url = current_url();
            session::with(|s| s.shield.toggle_site(&url));
            publish_shield();
            reload_active();
        }
        UiRequest::NewTab { url, container } => {
            let blank = url.is_none();
            let target = url.map(|u| normalize(&u)).unwrap_or_else(|| search::HOME.to_string());
            open_tab_in(&target, scoped(container).as_deref());
            publish_tabs();
            if blank {
                navigation::focus_address();
            }
        }
        UiRequest::SetTabContainer { id, container } => move_to_container(id, scoped(container)),
        UiRequest::WarmTab { id } => warm_tab(id),
        UiRequest::CloseDevTools => crate::devtools::undock(),
        UiRequest::ResizeExtensionPopup { dx, dy } => crate::overlay::resize_extension_popup(dx, dy),
        UiRequest::ResizeDevTools { dx } => crate::devtools::resize(dx),
        UiRequest::OpenDevTools { .. } => {
            if !crate::devtools::is_open() {
                crate::devtools::open_for_active();
            }
        }
        UiRequest::ForgetPermission { origin, permission } => {
            session::with(|s| echo_library::permissions::forget(&s.library, &origin, &permission));
            publish_permissions();
        }
        UiRequest::SelectTab { id } => select_tab(id),
        UiRequest::OpenTerminal => open_terminal(),
        UiRequest::OpenPage { page } => open_page(&page),
        UiRequest::AnswerPermission { id, allow, remember } => crate::permissions::answer(id, allow, remember),
        UiRequest::SleepTab { id } => sleep_tab(id),
        UiRequest::CloseTab { id } => close_tab(id),
        UiRequest::SetChromeWidth { pixels } => {
            let chrome = session::with(|s| s.chrome.clone()).flatten();
            crate::window::set_chrome_width(pixels as i32, chrome.as_ref());
            // Demande faite au chargement de la barre : elle apprend alors si la fenetre est etroite.
            publish(&CoreEvent::WindowNarrow { narrow: crate::window::is_narrow() });
        }
        UiRequest::SetSidebarCollapsed { collapsed } => {
            let chrome = session::with(|s| s.chrome.clone()).flatten();
            crate::window::set_collapsed(collapsed, chrome.as_ref());
        }
        UiRequest::RevealSidebar { reveal } => {
            let chrome = session::with(|s| s.chrome.clone()).flatten();
            crate::window::reveal_chrome(reveal, chrome.as_ref());
        }
        UiRequest::SetSpace { id } => crate::profiles::switch(&id),
        UiRequest::SetColorScheme { dark } => {
            let browsers = session::with(|s| s.tabs.browsers()).unwrap_or_default();
            crate::scheme::set(dark, &browsers);
            crate::system_theme::publish();
        }
        UiRequest::SetAccent { color } => {
            let host = session::with(|s| s.tabs.host()).flatten();
            crate::window::set_accent(&color, host.as_ref());
            if crate::roundness::enabled() {
                let script = crate::roundness::recolor_script(crate::window::accent_now());
                let frames = session::with(|s| s.tabs.main_frames()).unwrap_or_default();
                for frame in frames {
                    frame.execute_java_script(Some(&CefString::from(script.as_str())), Some(&CefString::from("echo://roundness")), 0);
                }
            }
            session::with(|s| {
                echo_library::settings::set(&s.library, "appearance.shell", &echo_library::settings::Value::Text(color.clone()))
            });
        }
        UiRequest::InstallExtension { source } => extensions::install_extension(&source),
        UiRequest::OpenCatalog => extensions::open_store(""),
        UiRequest::OpenExtensionPopup { id, anchor } => extensions::open_extension_popup(&id, anchor),
        UiRequest::OpenExtensionOptions { id } => extensions::open_extension_options(&id),
        UiRequest::RunContextMenu { action } => context::run(action),
        UiRequest::CloseContextMenu => crate::overlay::close_menu_from_page(),
        UiRequest::SetOverlayTheme { theme } => context::set_theme(theme),
        UiRequest::SetPageTheme { theme } => context::set_page_theme(theme),
        UiRequest::SetSchemeChoice { choice } => {
            if matches!(choice.as_str(), "light" | "dark" | "system") {
                publish(&CoreEvent::SchemeChoiceRequested { choice });
            }
        }
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
                extensions::publish_extensions();
            } else {
                extensions::remove_from_profile(&id);
            }
        }
        UiRequest::SetExtensionEnabled { id, enabled } => {
            let outcome = session::with(|s| s.extensions.set_enabled(&id, enabled));
            if let Some(Err(err)) = outcome {
                notify_error(&format!("changement impossible : {err}"));
            } else {
                mark_restart_needed();
            }
            extensions::publish_extensions();
        }
        UiRequest::ExitFullscreen => with_browser(|browser| {
            if let Some(host) = browser.host() {
                host.exit_fullscreen(1);
            }
        }),
        UiRequest::RestartBrowser => restart_browser(),
        UiRequest::InstallVideoCodecs => crate::codecs::install(),
        UiRequest::RemoveVideoCodecs => crate::codecs::remove(),
        UiRequest::DismissVideoCodecs { forever } => crate::codecs::dismiss(forever),
        UiRequest::CheckForUpdates => crate::update::check(true),
        UiRequest::AccountSignIn { email, password, create } => crate::account::sign_in(email, password, create),
        UiRequest::AccountSignOut => crate::account::sign_out(),
        UiRequest::AccountSync => {
            crate::account::sync_now();
        }
        UiRequest::AccountInspect => crate::account::inspect(),
        UiRequest::AccountDelete => crate::account::delete(),
        UiRequest::Find { text, forward, next } => crate::find::find(&text, forward, next),
        UiRequest::StopFind => crate::find::stop(),
        UiRequest::ImportSources => crate::importer::publish_sources(),
        UiRequest::ImportBrowser { id } => crate::importer::run(&id),
        UiRequest::FillForm { index } => crate::forms::fill(index as usize),
        UiRequest::ClearBrowsingData { since, history, cookies, cache } => {
            crate::privacy::clear_now(since, history, cookies, cache)
        }
        UiRequest::Suggest { query } => {
            let items = crate::suggest::for_address(&query);
            publish(&CoreEvent::Suggestions { query: query.clone(), items });
            crate::suggest::fetch_engine(query);
        }
        UiRequest::OpenSidebarSheet { sheet } => {
            if matches!(sheet.as_str(), "network" | "shield" | "extensions") {
                crate::window::reveal_chrome(true, session::with(|s| s.chrome.clone()).flatten().as_ref());
                publish(&CoreEvent::SidebarSheetRequested { sheet });
            }
        }
        UiRequest::ProfileForget { id, delete } => crate::profiles::forget(&id, delete),
        UiRequest::RoutineAccept { fingerprint, name } => crate::routines::accept(&fingerprint, &name),
        UiRequest::RoutineDismiss { fingerprint } => crate::routines::dismiss(&fingerprint),
        UiRequest::RoutineOpen { id } => crate::routines::open(id),
        UiRequest::RoutineRemove { id } => crate::routines::remove(id),
        UiRequest::NetworkWatch { on } => crate::network::watch(on),
        UiRequest::NetworkFocus { host } => crate::network::focus(host),
        UiRequest::NetworkBlockHost { host, blocked } => crate::network::block_host(&host, blocked),
        UiRequest::NetworkSetStrict { strict } => crate::network::set_strict(strict),
        UiRequest::AdminRefresh { query } => crate::account::admin_refresh(query),
        UiRequest::AdminSignOutAccount { id } => crate::account::admin_sign_out(&id),
        UiRequest::AdminDeleteAccount { id } => crate::account::admin_delete(&id),
        UiRequest::AdminSetFlag { id, admin } => crate::account::admin_set_flag(&id, admin),
        UiRequest::AdminAccountDetail { id } => crate::account::admin_detail(&id),

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
        UiRequest::UpdateSetting { key, value } => {
            library::update_setting(&key, &value);
            crate::account::setting_changed(&key);
            crate::signals::setting_changed(&key);
            crate::filtering::refresh_settings();
        }
        UiRequest::OpenExtensionManager => {
            open_tab(echo_extensions::profile::MANAGE_PAGE);
            publish_tabs();
        }
        UiRequest::SetTabMuted { id, muted } => {
            let host = session::with(|s| {
                let tab = s.tabs.get_mut(id)?;
                tab.muted = muted;
                tab.browser().and_then(|b| b.host())
            })
            .flatten();
            if let Some(host) = host {
                host.set_audio_muted(i32::from(muted));
            }
            publish_tabs();
        }
        UiRequest::KeepTabAwake { id, keep } => {
            session::with(|s| s.tabs.get_mut(id).map(|tab| tab.keep_awake = keep));
            if keep {
                warm_tab(id);
            }
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
        UiRequest::SetTabFolder { id, folder } => {
            session::with(|s| {
                if let Some(tab) = s.tabs.get_mut(id) {
                    tab.folder = folder;
                }
            });
            publish_tabs();
        }
        UiRequest::SetZoom { id, factor } => set_zoom(id, factor),
        UiRequest::MoveTab { id, to } => {
            session::with(|s| s.tabs.move_to(id, to));
            publish_tabs();
        }
    }
}

/// Referme ce qui est pose au-dessus de la page. A appeler des que le contenu change :
/// une fenetre d'extension qui survit a un changement d'onglet flotte dans le vide.
pub(super) fn dismiss_overlays() {
    crate::overlay::close_menu();
    if crate::overlay::open_popup_id().is_some() {
        crate::overlay::close_extension_popup();
        publish(&CoreEvent::ExtensionPopupChanged { id: None });
    }
}

/// Ouvre la fiche d'une extension dans un onglet, pour que Chromium mene l'installation.
///
/// Vrai quand une relance est necessaire pour que les extensions prennent effet.
pub(super) static RESTART_NEEDED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(super) fn mark_restart_needed() {
    RESTART_NEEDED.store(true, std::sync::atomic::Ordering::SeqCst);
}

pub(super) fn notify_error(message: &str) {
    warn!(%message, "demande refusee");
    publish(&CoreEvent::Notice {
        level: echo_contract::NoticeLevel::Error,
        message: message.to_string(),
        actions: Vec::new(),
    });
}

/// Enregistre les onglets, previent l'interface, puis relance apres un court delai
/// pour lui laisser le temps d'afficher son ecran d'attente.
pub fn restart_browser() {
    let snapshot = session::with(|s| s.tabs.to_snapshot());
    if let Some(snapshot) = snapshot {
        crate::restart::save(&crate::flags::data_dir(), &snapshot);
    }
    publish(&CoreEvent::Restarting {
        reason: "Redémarrage d’Echo".to_string(),
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
            tracing::info!("relance : sortie de la boucle demandee");
            crate::anchor::quit();
        }
    }
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
    let script = CefString::from(format!("window.__echoDeliver && window.__echoDeliver({payload})").as_str());
    let frame = session::with(|s| s.chrome_frame()).flatten();
    match frame {
        Some(frame) => frame.execute_java_script(Some(&script), None, 0),
        None => debug!("interface pas encore prete, evenement perdu"),
    }
    // Les pages pleine largeur ouvertes en onglet suivent le meme etat que la barre.
    let pages = session::with(|s| s.tabs.main_frames()).unwrap_or_default();
    for frame in pages.iter().filter(|f| CefString::from(&f.url()).to_string().starts_with(PAGES_URL)) {
        frame.execute_java_script(Some(&script), None, 0);
    }
}

/// Installation demandee par le bouton « Ajouter à Echo » du catalogue.
/// Un conteneur choisi dans l'interface appartient au profil affiche.
pub(super) fn scoped(container: Option<String>) -> Option<String> {
    let space = session::with(|s| s.tabs.space()).unwrap_or_else(|| crate::profiles::DEFAULT.to_string());
    // La navigation privee n'appartient a aucun profil : son contexte est en memoire, commun, jamais ecrit.
    container.map(|c| if c == crate::containers::PRIVATE { c } else { crate::profiles::scope_container(&space, &c) })
}

pub fn install_extension_from_store(page: &str) {
    extensions::install_extension(page);
}

/// Les pages pleine largeur d'Echo.
const PAGES_URL: &str = "echo://ui/pages.html";

/// Ouvre la page demandee, ou revient sur l'onglet qui la montre deja (dans le profil courant).
/// Ouvre une page pleine largeur d'Echo (raccourcis : Ctrl+H, Ctrl+J).
pub(crate) fn open_page_by_name(page: &str) {
    open_page(page);
}

fn open_page(page: &str) {
    if !matches!(page, "reglages" | "bibliotheque" | "extensions" | "bienvenue" | "admin" | "aide" | "effacer") {
        return;
    }
    let url = format!("{PAGES_URL}#{page}");
    let existing = session::with(|s| {
        let space = s.tabs.space();
        s.tabs.snapshot().into_iter().find(|t| t.url.starts_with(PAGES_URL) && t.space == space).map(|t| t.id)
    })
    .flatten();
    match existing {
        Some(id) => {
            select_tab(id);
            let browser = session::with(|s| s.tabs.get_mut(id).and_then(|t| t.browser())).flatten();
            if let Some(frame) = browser.and_then(|b| b.main_frame()) {
                frame.load_url(Some(&CefString::from(url.as_str())));
            }
        }
        None => {
            open_tab(&url);
            publish_tabs();
        }
    }
}

/// Recharge la page affichee : un changement du bouclier ne vaut qu'au prochain chargement.
fn reload_active() {
    let browser = session::with(|s| s.tabs.active().and_then(|tab| tab.browser())).flatten();
    // Sans cache : une ressource deja chargee reviendrait de la memoire sans repasser par le bouclier.
    if let Some(browser) = browser {
        browser.reload_ignore_cache();
    }
}
