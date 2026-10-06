//! Responsabilite : les outils de developpement ancres a droite de la page, dans la fenetre d'Echo.
//!
//! Chromium les ouvre d'ordinaire dans une fenetre a part, et CEF n'accepte pour eux qu'une vue de style
//! Chrome, qui plante dans notre fenetre sur mesure (mesure du 07/10, ChromeBrowserWidget::Init). Les
//! DevTools etant une application web, on charge donc leur interface (`devtools://devtools/bundled/…`)
//! dans une vue ordinaire ancree a droite, reliee a l'onglet par le port de debogage local de Chromium
//! (127.0.0.1 seulement). Les refermer retire la vue.

use cef::*;
use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::OnceLock;
use tracing::{info, warn};

/// Largeur du panneau, en pixels.
const PANEL_WIDTH: i32 = 560;

thread_local! {
    /// La vue des outils ancree, et l'identifiant de son navigateur.
    static DOCKED: RefCell<Option<(BrowserView, i32)>> = const { RefCell::new(None) };
}

wrap_browser_view_delegate! {
    pub struct DevToolsDelegate {
        marker: (),
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: PANEL_WIDTH, height: 600 }
        }
    }

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

/// Port de debogage local, choisi libre au demarrage. `ECHO_DEVTOOLS_PORT` l'impose.
pub fn port() -> u16 {
    static PORT: OnceLock<u16> = OnceLock::new();
    *PORT.get_or_init(|| {
        if let Some(port) = std::env::var("ECHO_DEVTOOLS_PORT").ok().and_then(|v| v.parse().ok()) {
            return port;
        }
        TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .and_then(|l| l.local_addr())
            .map(|a| a.port())
            .unwrap_or(9333)
    })
}

/// Identifiant de debogage de la page affichee a cette adresse (liste `/json/list` de Chromium).
fn target_for(url: &str) -> Option<String> {
    let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port())).ok()?;
    stream.set_read_timeout(Some(std::time::Duration::from_millis(800))).ok()?;
    stream.write_all(b"GET /json/list HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").ok()?;
    // Le serveur de Chromium peut garder la connexion ouverte : on lit jusqu'a la fin du JSON ou au delai.
    let mut raw = Vec::new();
    let mut chunk = [0u8; 16 * 1024];
    while let Ok(n) = stream.read(&mut chunk) {
        if n == 0 {
            break;
        }
        raw.extend_from_slice(&chunk[..n]);
        if raw.ends_with(b"]\n") || raw.ends_with(b"]") {
            break;
        }
    }
    let reply = String::from_utf8_lossy(&raw);
    let body = reply.split_once("\r\n\r\n")?.1;
    let list: serde_json::Value = serde_json::from_str(body).ok()?;
    list.as_array()?
        .iter()
        .find(|t| t["type"] == "page" && t["url"] == url)
        .and_then(|t| t["id"].as_str().map(str::to_string))
}

/// Ouvre les outils de l'onglet actif, ancres a droite. La liste des pages est demandee hors du fil
/// principal : c'est lui qui y repond, l'attendre dessus bloquerait tout.
pub fn open_for_active() {
    let url = crate::session::with(|s| s.tabs.active().map(|t| t.url.clone())).flatten().unwrap_or_default();
    std::thread::spawn(move || {
        let Some(target) = target_for(&url) else {
            warn!(%url, "page introuvable pour les outils de developpement");
            return;
        };
        let frontend =
            format!("devtools://devtools/bundled/devtools_app.html?ws=127.0.0.1:{}/devtools/page/{target}", port());
        crate::containers::later(move || create_and_dock(&frontend));
    });
}

fn create_and_dock(frontend: &str) {
    let mut client = crate::session::with(|s| s.client.clone()).flatten();
    let mut delegate = DevToolsDelegate::new(());
    let view = browser_view_create(
        client.as_mut(),
        Some(&CefString::from(frontend)),
        Some(&BrowserSettings::default()),
        None,
        None,
        Some(&mut delegate),
    );
    if let Some(view) = view {
        dock(view);
    }
}

fn window() -> Option<Window> {
    let chrome = crate::session::with(|s| s.chrome.clone()).flatten()?;
    View::from(&chrome).window()
}

/// Pose la vue des outils a droite de la page. Faux si la fenetre est introuvable.
pub fn dock(view: BrowserView) -> bool {
    undock();
    let Some(window) = window() else { return false };
    let id = view.browser().map(|b| b.identifier()).unwrap_or(-1);
    let mut as_view = View::from(&view);
    window.add_child_view(Some(&mut as_view));
    if let Some(layout) = window.get_layout().and_then(|l| l.as_box_layout()) {
        layout.set_flex_for_view(Some(&mut as_view), 0);
    }
    View::from(&window).invalidate_layout();
    DOCKED.with(|slot| *slot.borrow_mut() = Some((view, id)));
    info!("outils de developpement ancres dans la fenetre");
    true
}

/// Retire le panneau des outils, s'il est ouvert.
pub fn undock() {
    let Some((view, _)) = DOCKED.with(|slot| slot.borrow_mut().take()) else { return };
    if let Some(window) = window() {
        window.remove_child_view(Some(&mut View::from(&view)));
        View::from(&window).invalidate_layout();
    }
}

pub fn is_open() -> bool {
    DOCKED.with(|slot| slot.borrow().is_some())
}

/// Vrai si ce navigateur est celui des outils : sa fermeture ne concerne pas la fenetre.
pub fn owns(browser_id: i32) -> bool {
    DOCKED.with(|slot| slot.borrow().as_ref().is_some_and(|(_, id)| *id == browser_id))
}
