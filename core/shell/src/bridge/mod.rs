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

pub use library::publish_permissions;
pub use publish::{
    reset_tab_scroll, set_tab_dirty, set_tab_favicon, set_tab_scroll, take_pending_scroll,
    publish_filter_lists, publish_initial_state, publish_shield, publish_tab, publish_tabs,
    set_fullscreen, set_tab_title,
};
pub use navigation::{normalize, perform};
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
            let target = url.map(|u| normalize(&u)).unwrap_or_else(|| search::HOME.to_string());
            open_tab_in(&target, container.as_deref());
            publish_tabs();
        }
        UiRequest::SetTabContainer { id, container } => move_to_container(id, container),
        UiRequest::WarmTab { id } => warm_tab(id),
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
        UiRequest::AnswerPermission { id, allow, remember } => crate::permissions::answer(id, allow, remember),
        UiRequest::SleepTab { id } => sleep_tab(id),
        UiRequest::CloseTab { id } => close_tab(id),
        UiRequest::SetChromeWidth { pixels } => {
            let chrome = session::with(|s| s.chrome.clone()).flatten();
            crate::window::set_chrome_width(pixels as i32, chrome.as_ref());
        }
        UiRequest::SetSidebarCollapsed { collapsed } => {
            let chrome = session::with(|s| s.chrome.clone()).flatten();
            crate::window::set_collapsed(collapsed, chrome.as_ref());
        }
        UiRequest::RevealSidebar { reveal } => {
            let chrome = session::with(|s| s.chrome.clone()).flatten();
            crate::window::reveal_chrome(reveal, chrome.as_ref());
        }
        UiRequest::SetColorScheme { dark } => {
            let browsers = session::with(|s| s.tabs.browsers()).unwrap_or_default();
            crate::scheme::set(dark, &browsers);
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
        UiRequest::CloseContextMenu => context::close(),
        UiRequest::SetOverlayTheme { theme } => context::set_theme(theme),
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
            extensions::publish_extensions();
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
        other => debug!(?other, "demande pas encore traitee"),
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

/// Recharge la page affichee : un changement du bouclier ne vaut qu'au prochain chargement.
fn reload_active() {
    let browser = session::with(|s| s.tabs.active().and_then(|tab| tab.browser())).flatten();
    // Sans cache : une ressource deja chargee reviendrait de la memoire sans repasser par le bouclier.
    if let Some(browser) = browser {
        browser.reload_ignore_cache();
    }
}
