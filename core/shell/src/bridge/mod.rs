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
use tracing::{debug, warn};

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
        UiRequest::GoBack { .. } => with_browser(|browser| browser.go_back()),
        UiRequest::GoForward { .. } => with_browser(|browser| browser.go_forward()),
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
        UiRequest::SetChromeHeight { pixels } => {
            let chrome = session::with(|s| s.chrome.clone()).flatten();
            crate::window::set_chrome_height(pixels as i32, chrome.as_ref());
        }
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
