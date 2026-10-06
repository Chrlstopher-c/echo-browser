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
use std::sync::atomic::{AtomicI32, Ordering};
use tracing::{info, warn};

/// Largeur du panneau, en pixels, reglable a la poignee.
static WIDTH: AtomicI32 = AtomicI32::new(560);
const MIN_WIDTH: i32 = 300;
/// Place minimale laissee a la page quand on elargit les outils.
const MIN_PAGE: i32 = 320;
const BAR_HEIGHT: i32 = 30;
const GRIP_WIDTH: i32 = 10;

/// Le panneau ancre : son conteneur (barre + outils), les identifiants de ses navigateurs, la poignee.
struct Docked {
    container: Panel,
    browsers: Vec<i32>,
    grip: Option<crate::overlay::Overlay>,
}

thread_local! {
    static DOCKED: RefCell<Option<Docked>> = const { RefCell::new(None) };
}

wrap_browser_view_delegate! {
    pub struct DevToolsDelegate {
        marker: (),
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: WIDTH.load(Ordering::Relaxed), height: 600 }
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

wrap_panel_delegate! {
    struct ContainerDelegate {
        marker: (),
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: WIDTH.load(Ordering::Relaxed), height: 600 }
        }
    }

    impl PanelDelegate {}
}

wrap_browser_view_delegate! {
    struct BarDelegate {
        marker: (),
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: WIDTH.load(Ordering::Relaxed), height: BAR_HEIGHT }
        }
    }

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

fn browser_view(url: &str, delegate: &mut BrowserViewDelegate) -> Option<BrowserView> {
    let mut client = crate::session::with(|s| s.client.clone()).flatten();
    browser_view_create(client.as_mut(), Some(&CefString::from(url)), Some(&BrowserSettings::default()), None, None, Some(delegate))
}

fn create_and_dock(frontend: &str) {
    let (Some(bar), Some(tools)) = (
        browser_view("echo://ui/outils-barre.html", &mut BarDelegate::new(())),
        browser_view(frontend, &mut DevToolsDelegate::new(())),
    ) else {
        return warn!("vues des outils de developpement non creees");
    };
    let Some(container) = panel_create(Some(&mut ContainerDelegate::new(()))) else { return };
    let settings = BoxLayoutSettings { horizontal: 0, cross_axis_alignment: AxisAlignment::STRETCH, ..Default::default() };
    let layout = container.set_to_box_layout(Some(&settings));
    container.add_child_view(Some(&mut View::from(&bar)));
    let mut tools_view = View::from(&tools);
    container.add_child_view(Some(&mut tools_view));
    if let Some(layout) = layout {
        layout.set_flex_for_view(Some(&mut tools_view), 1);
    }
    let browsers = [&bar, &tools].iter().filter_map(|v| v.browser()).map(|b| b.identifier()).collect();
    dock(container, browsers);
}

fn window() -> Option<Window> {
    let chrome = crate::session::with(|s| s.chrome.clone()).flatten()?;
    View::from(&chrome).window()
}

fn dock(container: Panel, browsers: Vec<i32>) {
    undock();
    let Some(window) = window() else { return };
    let mut as_view = View::from(&container);
    window.add_child_view(Some(&mut as_view));
    if let Some(layout) = window.get_layout().and_then(|l| l.as_box_layout()) {
        layout.set_flex_for_view(Some(&mut as_view), 0);
    }
    window.layout();
    DOCKED.with(|slot| *slot.borrow_mut() = Some(Docked { container, browsers, grip: None }));
    place();
    info!("outils de developpement ancres dans la fenetre");
}

/// Pose la poignee dans l'espace entre la page et les outils. A rappeler a chaque disposition.
pub fn place() {
    let Some(bounds) = DOCKED.with(|slot| slot.borrow().as_ref().map(|d| View::from(&d.container).bounds())) else {
        return;
    };
    let rect = Rect { x: bounds.x - GRIP_WIDTH, y: bounds.y, width: GRIP_WIDTH, height: bounds.height };
    let has_grip = DOCKED.with(|slot| slot.borrow().as_ref().is_some_and(|d| d.grip.is_some()));
    if has_grip {
        DOCKED.with(|slot| {
            if let Some(grip) = slot.borrow().as_ref().and_then(|d| d.grip.as_ref()) {
                grip.set_bounds(rect);
            }
        });
        return;
    }
    let Some(chrome) = crate::session::with(|s| s.chrome.clone()).flatten() else { return };
    let grip = crate::overlay::Overlay::open(&chrome, "echo://ui/outils-poignee.html", rect);
    DOCKED.with(|slot| {
        if let Some(docked) = slot.borrow_mut().as_mut() {
            docked.grip = grip;
        }
    });
}

/// La poignee a ete tiree de `dx` pixels (vers la gauche : negatif, le panneau s'elargit).
pub fn resize(dx: i32) {
    let Some(window) = window() else { return };
    let available = View::from(&window).bounds().width;
    let max = (available - MIN_PAGE).max(MIN_WIDTH);
    let next = (WIDTH.load(Ordering::Relaxed) - dx).clamp(MIN_WIDTH, max);
    WIDTH.store(next, Ordering::Relaxed);
    DOCKED.with(|slot| {
        if let Some(docked) = slot.borrow().as_ref() {
            View::from(&docked.container).invalidate_layout();
        }
    });
    window.layout();
    place();
}

/// Retire le panneau des outils, s'il est ouvert.
pub fn undock() {
    let Some(docked) = DOCKED.with(|slot| slot.borrow_mut().take()) else { return };
    if let Some(grip) = docked.grip {
        grip.close();
    }
    if let Some(window) = window() {
        window.remove_child_view(Some(&mut View::from(&docked.container)));
        View::from(&window).invalidate_layout();
    }
}

pub fn is_open() -> bool {
    DOCKED.with(|slot| slot.borrow().is_some())
}

/// Vrai si ce navigateur appartient au panneau : sa fermeture ne concerne pas la fenetre.
pub fn owns(browser_id: i32) -> bool {
    DOCKED.with(|slot| slot.borrow().as_ref().is_some_and(|d| d.browsers.contains(&browser_id)))
}
