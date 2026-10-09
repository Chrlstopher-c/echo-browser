//! Responsabilite : le point d'ancrage des extensions. Les API d'extension de Chrome (`tabs.create`,
//! `windows.create`, `tabs.query`…) exigent une vraie fenetre Chrome ; la fenetre sur mesure d'Echo n'en
//! est pas une et elles echouaient (« No current window »). On cree donc une fenetre Chrome jamais
//! affichee. Les onglets que les extensions y ouvrent sont aussitot repris comme onglets d'Echo.

use cef::*;
use std::cell::RefCell;
use tracing::{debug, info};

thread_local! {
    /// La fenetre d'ancrage et l'identifiant de son navigateur d'origine (garde ouvert : sans lui,
    /// fermer le dernier onglet repris fermerait la fenetre).
    static ANCHOR: RefCell<Option<(Window, i32)>> = const { RefCell::new(None) };
}

wrap_browser_view_delegate! {
    struct AnchorViewDelegate {
        marker: (),
    }

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::CHROME
        }

        fn chrome_toolbar_type(&self, _browser_view: Option<&mut BrowserView>) -> ChromeToolbarType {
            ChromeToolbarType::NONE
        }
    }
}

wrap_window_delegate! {
    struct AnchorWindowDelegate {
        view: RefCell<Option<BrowserView>>,
    }

    impl ViewDelegate {}

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            let (Some(window), Some(view)) = (window, self.view.borrow().clone()) else { return };
            window.add_child_view(Some(&mut View::from(&view)));
            let id = view.browser().map(|b| b.identifier()).unwrap_or(-1);
            ANCHOR.with(|slot| *slot.borrow_mut() = Some((window.clone(), id)));
            info!(id, "ancrage des extensions pret (fenetre jamais affichee)");
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::CHROME
        }

        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            1
        }
    }
}

/// Cree la fenetre d'ancrage, sans l'afficher.
pub fn create(client: Option<&mut Client>) {
    let mut delegate = AnchorViewDelegate::new(());
    let view = browser_view_create(
        client,
        Some(&CefString::from("about:blank")),
        Some(&BrowserSettings::default()),
        None,
        None,
        Some(&mut delegate),
    );
    let mut window_delegate = AnchorWindowDelegate::new(RefCell::new(view));
    window_create_top_level(Some(&mut window_delegate));
}

/// Vrai si ce navigateur est celui d'origine de l'ancrage.
pub fn is_anchor(browser_id: i32) -> bool {
    ANCHOR.with(|slot| slot.borrow().as_ref().is_some_and(|(_, id)| *id == browser_id))
}

/// Un navigateur ne d'une API d'extension (onglet de la fenetre d'ancrage) : on recupere son adresse,
/// on le ferme et on l'ouvre comme un vrai onglet d'Echo. L'adresse n'est connue qu'apres le depart de
/// la navigation : on la relit un instant plus tard.
pub fn adopt_foreign(browser: Browser) {
    let id = browser.identifier();
    let browser = std::sync::Arc::new(std::sync::Mutex::new(Some(browser)));
    let take = move || {
        // Le navigateur d'origine de l'ancrage peut naitre avant d'etre enregistre : on le relit ici.
        if is_anchor(id) {
            return;
        }
        let Some(browser) = browser.lock().ok().and_then(|mut b| b.take()) else { return };
        let url = browser.main_frame().map(|f| CefString::from(&f.url()).to_string()).unwrap_or_default();
        debug!(id, %url, "onglet d'extension repris");
        if let Some(host) = browser.host() {
            host.close_browser(1);
        }
        if !url.is_empty() && url != "about:blank" {
            crate::bridge::open_tab_like_active(&url);
            crate::bridge::publish_tabs();
        }
    };
    crate::containers::later(move || crate::containers::later(take));
}

/// Libere la fenetre d'ancrage avant l'arret.
pub fn release() {
    if let Some((window, _)) = ANCHOR.with(|slot| slot.borrow_mut().take()) {
        window.close();
    }
}

/// Sort de la boucle de messages. La fenetre d'ancrage est de style Chrome : tant qu'elle vit, Chromium garde
/// la boucle en marche et `quit_message_loop` reste sans effet (relance qui tourne a vide, processus fantome
/// apres fermeture). On la ferme donc d'abord.
pub fn quit() {
    release();
    quit_message_loop();
}
