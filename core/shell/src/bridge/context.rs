//! Responsabilite : le menu contextuel — ce que le clic droit ouvre, et ce qu'il declenche.

use super::{navigation, notify_error, publish, session};
use crate::menu::{self, Click};
use cef::{
    Browser, CefString, ImplBrowser, ImplBrowserHost, ImplFrame, Rect,
};
use echo_contract::{ContextTarget, CoreEvent, MenuItemKind};
use tracing::{debug, warn};

thread_local! {
    /// Les couleurs de l'espace courant, posees par l'interface. Sans elles, le menu
    /// s'afficherait dans la palette par defaut, etranger a la barre.
    static THEME: std::cell::RefCell<echo_contract::OverlayTheme> =
        std::cell::RefCell::new(echo_contract::OverlayTheme::default());

    /// Ce sur quoi on a clique en dernier. L'action arrive apres, par un autre message :
    /// sans cette memoire, « copier l'adresse du lien » ne saurait plus de quel lien.
    static TARGET: std::cell::RefCell<Option<Click>> = const { std::cell::RefCell::new(None) };

    /// Point du clic droit dans la page : « Examiner l'element » y selectionne l'element.
    static CLICK_POINT: std::cell::Cell<(i32, i32)> = const { std::cell::Cell::new((0, 0)) };
}

/// Retient les couleurs que porteront les surimpressions.
pub fn set_theme(theme: echo_contract::OverlayTheme) {
    THEME.with(|cell| *cell.borrow_mut() = theme);
}

fn theme() -> echo_contract::OverlayTheme {
    THEME.with(|cell| cell.borrow().clone())
}

/// Retient la cible et demande l'ouverture du menu au-dessus de la page.
pub fn open(click: Click, x: i32, y: i32) {
    let facts = menu::PageFacts {
        hidden_here: crate::page_memory::active_has_hidden(),
        watched: crate::watch::active_watched(),
        forms: crate::forms::names(),
    };
    let target = menu::build(&click, facts);
    let (width, height) = menu::size_of(&target);
    TARGET.with(|cell| *cell.borrow_mut() = Some(click));
    CLICK_POINT.with(|cell| cell.set((x, y)));

    let Some(chrome) = session::with(|s| s.chrome.clone()).flatten() else { return };
    // Le clic est repere dans la page ; la surimpression, dans la fenetre. Le decalage
    // est celui du conteneur de la page, lu sur lui plutot que devine.
    let origine = session::with(|s| s.tabs.host())
        .flatten()
        .map(|host| cef::ImplView::bounds(&cef::View::from(&host)))
        .unwrap_or(Rect { x: 0, y: 0, width: 0, height: 0 });
    let anchor = Rect { x: x + origine.x, y: y + origine.y, width, height };
    let payload = serde_json::json!({ "target": target, "theme": theme() }).to_string();
    crate::overlay::open_menu(&chrome, &payload, anchor);
    debug!(entrees = target.entries.len(), x, y, "menu contextuel ouvert");
}

/// Referme le menu sans rien declencher.
pub fn close() {
    crate::overlay::close_menu();
}

/// Execute l'action choisie, puis referme le menu.
pub fn run(action: MenuItemKind) {
    close();
    let click = TARGET.with(|cell| cell.borrow().as_ref().map(clone_click));
    let Some(click) = click else {
        warn!(?action, "action de menu sans cible");
        return;
    };
    match action {
        MenuItemKind::OpenLinkInTab | MenuItemKind::OpenImage => open_and_show(&target_url(&click, action)),
        MenuItemKind::OpenLinkInBackground => super::open_tab_like_active(&click.link),
        MenuItemKind::OpenSelection => open_and_show(&navigation::normalize(&click.selection)),
        MenuItemKind::SearchSelection => open_and_show(&crate::search::query_url(&click.selection)),
        MenuItemKind::CopyLink => copy(&click.link),
        MenuItemKind::CopyImageLink => copy(&click.image),
        MenuItemKind::CopyPageLink => copy(&click.page),
        MenuItemKind::SaveLink => download(&click.link),
        MenuItemKind::SaveImage => download(&click.image),
        MenuItemKind::CopyImage => crate::clipboard::copy_image(&click.image),
        MenuItemKind::OpenMedia => open_and_show(&click.media),
        MenuItemKind::CopyMediaLink => copy(&click.media),
        MenuItemKind::SaveMedia => download(&click.media),
        MenuItemKind::Copy => with_page(|frame| frame.copy()),
        MenuItemKind::Cut => with_page(|frame| frame.cut()),
        MenuItemKind::Paste => with_page(|frame| frame.paste()),
        MenuItemKind::PastePlain => with_page(|frame| frame.paste_and_match_style()),
        MenuItemKind::SelectAll => with_page(|frame| frame.select_all()),
        MenuItemKind::Back => navigation::travel(false),
        MenuItemKind::Forward => navigation::travel(true),
        MenuItemKind::Reload => with_browser_host(|browser| browser.reload()),
        MenuItemKind::Print => with_browser_host(|browser| {
            if let Some(host) = browser.host() {
                host.print();
            }
        }),
        // `view_source` de CEF ouvre une fenetre a part : le code source s'affiche dans un onglet.
        MenuItemKind::ViewSource => open_and_show(&format!("view-source:{}", click.page)),
        MenuItemKind::Inspect => inspect(),
        MenuItemKind::HideElement => hide_element(),
        MenuItemKind::UnhideElements => crate::page_memory::unhide_active(),
        MenuItemKind::WatchPage => crate::watch::watch_active(),
        MenuItemKind::FillForm1 => crate::forms::fill(0),
        MenuItemKind::FillForm2 => crate::forms::fill(1),
        MenuItemKind::FillForm3 => crate::forms::fill(2),
        MenuItemKind::ManageForms => super::open_page_by_name("reglages"),
        MenuItemKind::UnwatchPage => crate::watch::unwatch_active(),
        MenuItemKind::Bookmark => {
            let active = session::with(|s| s.tabs.active_id()).flatten();
            if let Some(id) = active {
                super::library::add_bookmark(id);
            }
        }
        MenuItemKind::ToggleShield => {
            let url = navigation::current_url();
            session::with(|s| s.shield.toggle_site(&url));
            super::publish::publish_shield();
        }
        MenuItemKind::SavePage => download(&click.page),
        MenuItemKind::Separator => {}
    }
}

fn target_url(click: &Click, action: MenuItemKind) -> String {
    match action {
        MenuItemKind::OpenImage => click.image.clone(),
        _ => click.link.clone(),
    }
}

fn open_and_show(url: &str) {
    if url.is_empty() {
        return;
    }
    super::open_tab_like_active(url);
    super::publish::publish_tabs();
}

/// Le presse-papiers passe par la page : le processus navigateur n'y a pas acces
/// directement, mais la page selectionnee, elle, sait copier.
fn copy(value: &str) {
    if value.is_empty() {
        return;
    }
    let script = format!(
        "navigator.clipboard.writeText({}).catch(()=>{{}});",
        serde_json::to_string(value).unwrap_or_else(|_| "''".to_string())
    );
    let frame = session::with(|s| s.chrome_frame()).flatten();
    match frame {
        Some(frame) => {
            frame.execute_java_script(Some(&CefString::from(script.as_str())), None, 0);
            publish(&CoreEvent::Notice {
                level: echo_contract::NoticeLevel::Info,
                message: "Copié.".to_string(),
            });
        }
        None => notify_error("copie impossible : interface injoignable"),
    }
}

fn download(url: &str) {
    if url.is_empty() {
        return;
    }
    let browser = session::with(|s| s.tabs.active().and_then(|tab| tab.browser())).flatten();
    match browser.and_then(|browser| browser.host()) {
        Some(host) => host.start_download(Some(&CefString::from(url))),
        None => notify_error("téléchargement impossible : aucune page active"),
    }
}

/// F12 : ouvre les outils de developpement de l'onglet actif, ou les referme.
pub fn toggle_devtools() {
    let browser = session::with(|s| s.tabs.active().and_then(|tab| tab.browser())).flatten();
    let Some(host) = browser.and_then(|browser| browser.host()) else { return };
    let _ = host;
    if crate::devtools::is_open() {
        crate::devtools::undock();
    } else {
        crate::devtools::open_for_active();
    }
}

/// Ouvre l'inspecteur sur l'element clique : la page le retient avant que les outils ne la retrecissent.
fn inspect() {
    let (x, y) = CLICK_POINT.with(std::cell::Cell::get);
    let zoom = session::with(|s| s.tabs.active().map(|tab| tab.zoom)).flatten().unwrap_or(1.0).max(0.25);
    let mark = format!("window.__echoInspect=document.elementFromPoint({}, {})", x as f32 / zoom, y as f32 / zoom);
    with_page(|frame| frame.execute_java_script(Some(&CefString::from(mark.as_str())), None, 0));
    if !crate::devtools::is_open() {
        crate::devtools::open_for_active();
    }
    crate::devtools::reveal_marked();
}

/// Masque l'element sous le clic droit ; la page annonce gabarit et selecteur, retenus par `page_memory`.
fn hide_element() {
    let (x, y) = CLICK_POINT.with(std::cell::Cell::get);
    let zoom = session::with(|s| s.tabs.active().map(|tab| tab.zoom)).flatten().unwrap_or(1.0).max(0.25);
    let script = crate::page_memory::hide_script(x as f32 / zoom, y as f32 / zoom);
    if let Some(browser) = session::with(|s| s.tabs.active().and_then(|t| t.browser())).flatten() {
        crate::page_memory::arm(browser.identifier());
    }
    with_page(|frame| frame.execute_java_script(Some(&CefString::from(script.as_str())), None, 0));
}

fn with_page(action: impl FnOnce(&cef::Frame)) {
    match session::with(|s| s.active_frame()).flatten() {
        Some(frame) => action(&frame),
        None => warn!("aucune page active : action de menu sans effet"),
    }
}

fn with_browser_host(action: impl FnOnce(&Browser)) {
    let browser = session::with(|s| s.tabs.active().and_then(|tab| tab.browser())).flatten();
    match browser {
        Some(browser) => action(&browser),
        None => warn!("aucune page active : action de menu sans effet"),
    }
}

fn clone_click(click: &Click) -> Click {
    Click {
        link: click.link.clone(),
        image: click.image.clone(),
        media: click.media.clone(),
        selection: click.selection.clone(),
        page: click.page.clone(),
        editable: click.editable,
        can_go_back: click.can_go_back,
        can_go_forward: click.can_go_forward,
    }
}

/// Le menu tel qu'il sera affiche, pour les essais et le journal.
pub fn preview(click: &Click) -> ContextTarget {
    menu::build(click, menu::PageFacts::default())
}
