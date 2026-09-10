//! Responsabilite : le canal entre l'interface et le coeur, dans les deux sens.
//!
//! L'interface poste ses demandes au schema interne ; elles sont mises en file et
//! rejouees sur le thread interface, seul endroit ou les objets Chromium sont manipulables.
//! Les evenements repartent par un appel de fonction dans la page.

pub mod script;

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use echo_contract::{CoreEvent, ShieldView, TabView, UiRequest};
use parking_lot::Mutex;
use std::collections::VecDeque;
use tracing::{debug, warn};

use crate::session;

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
        other => debug!(?other, "demande pas encore traitee"),
    }
}

fn with_browser(action: impl FnOnce(&Browser)) {
    let browser = session::with(|s| s.content.as_ref().and_then(|view| view.browser())).flatten();
    match browser {
        Some(browser) => action(&browser),
        None => warn!("aucune vue de contenu : demande sans effet"),
    }
}

fn navigate(url: &str) {
    let frame = session::with(|s| s.content_frame()).flatten();
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
    session::with(|s| s.content_frame().map(|frame| CefString::from(&frame.url()).to_string()))
        .flatten()
        .unwrap_or_default()
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

/// Diffuse l'etat de l'onglet courant.
pub fn publish_tab(url: &str, title: &str, loading: bool) {
    let (can_go_back, can_go_forward) = session::with(|s| {
        s.content
            .as_ref()
            .and_then(|view| view.browser())
            .map(|browser| (browser.can_go_back() == 1, browser.can_go_forward() == 1))
            .unwrap_or((false, false))
    })
    .unwrap_or((false, false));

    publish(&CoreEvent::TabUpdated {
        tab: TabView {
            id: 0,
            title: title.to_string(),
            url: url.to_string(),
            loading,
            progress: if loading { 0.5 } else { 1.0 },
            can_go_back,
            can_go_forward,
            favicon: None,
        },
    });
}
