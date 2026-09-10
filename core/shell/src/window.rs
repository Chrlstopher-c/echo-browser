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
        content_host: RefCell<Option<Panel>>,
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
            // Disposition verticale : la bande d'interface en haut a sa hauteur preferee,
            // la vue web dessous prend tout le reste (flex 1).
            let layout = window.set_to_box_layout(Some(&vertical_layout()));
            let mut chrome_view = View::from(chrome);
            window.add_child_view(Some(&mut chrome_view));

            let host = self.content_host.borrow().clone();
            if let Some(host) = host.as_ref() {
                let mut host_view = View::from(host);
                window.add_child_view(Some(&mut host_view));
                if let Some(layout) = layout {
                    layout.set_flex_for_view(Some(&mut host_view), 1);
                }
            }

            window.set_title(Some(&CefString::from(WINDOW_TITLE)));
            window.show();
            info!(contenu = host.is_some(), "fenetre du navigateur affichee");
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            *self.chrome_view.borrow_mut() = None;
            *self.content_host.borrow_mut() = None;
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
        preferred_height: i32,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: INITIAL_WIDTH, height: self.preferred_height }
        }
    }

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            self.runtime_style
        }
    }
}

/// Disposition verticale de la fenetre.
fn vertical_layout() -> BoxLayoutSettings {
    BoxLayoutSettings {
        horizontal: 0,
        main_axis_alignment: AxisAlignment::START,
        cross_axis_alignment: AxisAlignment::STRETCH,
        default_flex: 0,
        ..Default::default()
    }
}

/// Cree une vue navigateur en style Alloy, seul style compatible avec une fenetre sur mesure.
pub fn create_view(client: Option<&mut Client>, url: &str, preferred_height: i32) -> Option<BrowserView> {
    let mut delegate = ChromeViewDelegate::new(RuntimeStyle::ALLOY, preferred_height);
    browser_view_create(
        client,
        Some(&CefString::from(url)),
        Some(&BrowserSettings::default()),
        None,
        None,
        Some(&mut delegate),
    )
}

/// Cree la vue qui porte l'interface du navigateur.
pub fn create_chrome_view(client: Option<&mut Client>, url: &str) -> Option<BrowserView> {
    // Style Alloy impose : une vue en style Chrome ajoutee a une fenetre sur mesure cherche
    // l'infrastructure d'onglets du vrai Chrome et fait planter le processus dans
    // tabs::TabInterface::GetFromContents. Mesure le 2026-09-10, pile a l'appui.
    create_view(client, url, CHROME_HEIGHT)
}
