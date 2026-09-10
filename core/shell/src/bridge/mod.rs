//! Responsabilite : le canal entre l'interface et le coeur, dans les deux sens.
//!
//! L'interface poste ses demandes au schema interne ; elles sont mises en file et
//! rejouees sur le thread interface, seul endroit ou les objets Chromium sont manipulables.
//! Les evenements repartent par un appel de fonction dans la page.

pub mod script;

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use echo_contract::{CoreEvent, ShieldView, TabId, UiRequest};
use parking_lot::Mutex;
use std::collections::VecDeque;
use tracing::{debug, info, warn};

use crate::session;

/// Page ouverte dans un nouvel onglet.
const HOME_URL: &str = "https://www.qwant.com/";

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
            let target = url.map(|u| normalize(&u)).unwrap_or_else(|| HOME_URL.to_string());
            open_tab(&target);
            publish_tabs();
        }
        UiRequest::SelectTab { id } => {
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
        UiRequest::InstallExtension { source } => {
            let outcome = session::with(|s| s.extensions.install(&source));
            let (ok, reason) = match outcome {
                Some(Ok(extension)) => {
                    info!(nom = %extension.name, "extension installee");
                    mark_restart_needed();
                    (true, None)
                }
                Some(Err(err)) => {
                    warn!(%err, "installation impossible");
                    (false, Some(err.to_string()))
                }
                None => (false, Some("navigateur indisponible".to_string())),
            };
            publish_extensions();
            publish(&CoreEvent::InstallFinished { source, ok, reason });
        }
        UiRequest::RemoveExtension { id } => {
            let outcome = session::with(|s| s.extensions.remove(&id));
            if let Some(Err(err)) = outcome {
                notify_error(&format!("suppression impossible : {err}"));
            } else {
                mark_restart_needed();
            }
            publish_extensions();
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
        UiRequest::RestartBrowser => restart_browser(),
        other => debug!(?other, "demande pas encore traitee"),
    }
}

/// Ouvre un onglet. Chaque appel a Chromium se fait hors de l'acces a l'etat : la creation
/// de la vue et son rattachement declenchent des rappels qui veulent lire cet etat.
pub fn open_tab(url: &str) {
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
        .map(|extension| echo_contract::ExtensionView {
            id: extension.id,
            name: extension.name,
            version: extension.version,
            enabled: extension.enabled,
            pending,
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

/// Applique un raccourci clavier.
pub fn perform(action: crate::shortcuts::Action) {
    use crate::shortcuts::Action;
    match action {
        Action::NewTab => {
            open_tab(HOME_URL);
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
        Action::FocusAddress => publish(&CoreEvent::Notice {
            level: echo_contract::NoticeLevel::Info,
            message: "focus-address".to_string(),
        }),
        Action::ToggleFullscreen => debug!("plein ecran : pas encore traite"),
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
    format!("https://www.qwant.com/?q={}", urlencode(trimmed))
}

fn urlencode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            b' ' => "+".to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
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
    let changed = session::with(|s| {
        let tab = s.tabs.by_browser(browser_id)?;
        tab.url = url.to_string();
        tab.title = if title.is_empty() { url.to_string() } else { title.to_string() };
        tab.loading = loading;
        if !loading {
            tab.record_visit(url);
        }
        Some(())
    })
    .flatten();
    if changed.is_none() {
        return;
    }
    publish_tabs();
}

/// Diffuse la liste complete des onglets et celui qui est actif.
pub fn publish_tabs() {
    let Some((tabs, active)) = session::with(|s| (s.tabs.snapshot(), s.tabs.active_id())) else {
        return;
    };
    publish(&CoreEvent::TabsChanged { tabs, active });
}
