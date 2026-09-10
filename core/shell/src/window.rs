//! Responsabilite : la fenetre du navigateur et la place respective de l'interface et du contenu.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use std::cell::RefCell;
use tracing::info;

/// Hauteur de la bande d'interface au repos. L'interface peut en reclamer davantage
/// quand elle ouvre un panneau — voir `UiRequest::SetChromeHeight`.
pub const CHROME_HEIGHT: i32 = 78;

/// Titre porte par la fenetre. Le nom du projet ne s'affiche nulle part dans l'application.
const WINDOW_TITLE: &str = "Navigateur";

const INITIAL_WIDTH: i32 = 1440;
const INITIAL_HEIGHT: i32 = 900;

wrap_window_delegate! {
    pub struct BrowserWindowDelegate {
        chrome_view: RefCell<Option<BrowserView>>,
        runtime_style: RuntimeStyle,
        initial_show_state: ShowState,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: INITIAL_WIDTH, height: INITIAL_HEIGHT }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            // L'emprunt est relache avant add_child_view : CEF rappelle le delegue pendant
            // cet appel, et un emprunt encore tenu ferait paniquer le thread interface.
            let chrome = self.chrome_view.borrow().clone();
            let (Some(window), Some(chrome)) = (window, chrome.as_ref()) else {
                return;
            };
            let mut view = View::from(chrome);
            window.add_child_view(Some(&mut view));
            window.set_title(Some(&CefString::from(WINDOW_TITLE)));
            window.show();
            info!("fenetre du navigateur affichee");
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            *self.chrome_view.borrow_mut() = None;
        }

        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            let chrome = self.chrome_view.borrow();
            let Some(browser) = chrome.as_ref().and_then(|view| view.browser()) else {
                return 1;
            };
            match browser.host() {
                Some(host) => host.try_close_browser(),
                None => 1,
            }
        }

        fn initial_show_state(&self, _window: Option<&mut Window>) -> ShowState {
            self.initial_show_state
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            self.runtime_style
        }
    }
}

wrap_browser_view_delegate! {
    pub struct ChromeViewDelegate {
        runtime_style: RuntimeStyle,
    }

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            self.runtime_style
        }
    }
}

/// Cree la vue qui porte l'interface du navigateur.
pub fn create_chrome_view(client: Option<&mut Client>, url: &str) -> Option<BrowserView> {
    // Style Alloy impose : une vue en style Chrome ajoutee a une fenetre sur mesure cherche
    // l'infrastructure d'onglets du vrai Chrome et fait planter le processus dans
    // tabs::TabInterface::GetFromContents. Mesure le 2026-09-10, pile a l'appui.
    let mut delegate = ChromeViewDelegate::new(RuntimeStyle::ALLOY);
    browser_view_create(
        client,
        Some(&CefString::from(url)),
        Some(&BrowserSettings::default()),
        None,
        None,
        Some(&mut delegate),
    )
}
