//! Responsabilite : la fenetre du navigateur et la place respective de l'interface et du contenu.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicI32, Ordering};
use tracing::{debug, info};

/// Largeur de la barre laterale au repos.
pub const CHROME_WIDTH: i32 = 240;

/// Largeur minimale une fois la barre repliee.
const MIN_CHROME_WIDTH: i32 = 0;

/// Garde-fou : une barre qui prendrait toute la fenetre masquerait la page.
const MAX_CHROME_WIDTH: i32 = 520;

/// Marge entre la page et les bords de la fenetre. C'est elle qui fait flotter la page.
const CONTENT_INSET: i32 = 8;

/// Largeur courante de la barre laterale. Partagee parce que la disposition l'interroge
/// depuis un rappel de Chromium, hors de tout acces a l'etat.
static CHROME_WIDTH_NOW: AtomicI32 = AtomicI32::new(CHROME_WIDTH);

/// Fixe la largeur reclamee par la barre laterale et relance la disposition.
pub fn set_chrome_width(pixels: i32, chrome: Option<&BrowserView>) {
    let clamped = pixels.clamp(MIN_CHROME_WIDTH, MAX_CHROME_WIDTH);
    if CHROME_WIDTH_NOW.swap(clamped, Ordering::Relaxed) == clamped {
        return;
    }
    if let Some(chrome) = chrome {
        let view = View::from(chrome);
        view.invalidate_layout();
        if let Some(parent) = view.parent_view() {
            parent.invalidate_layout();
        }
    }
    debug!(largeur = clamped, "largeur de la barre laterale");
}

/// Titre porte par la fenetre. Le nom du projet ne s'affiche nulle part dans l'application.
const WINDOW_TITLE: &str = "Navigateur";

/// Teinte posee avant que l'interface ne se charge : sans elle, la marge autour de la page
/// est noire pendant tout le demarrage, puis saute a la couleur de l'espace. C'est la valeur
/// plate de l'espace par defaut (`shell` de « graphite », ui/src/spaces/space-palette.ts).
const DEFAULT_SHELL: u32 = 0xFF14_1517;

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
            let layout = window.set_to_box_layout(Some(&side_by_side_layout()));
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

            View::from(&*window).set_background_color(DEFAULT_SHELL);
            window.set_title(Some(&CefString::from(WINDOW_TITLE)));
            window.show();
            info!(contenu = host.is_some(), "fenetre du navigateur affichee");
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            *self.chrome_view.borrow_mut() = None;
            *self.content_host.borrow_mut() = None;
            info!("fenetre fermee, arret du navigateur");
            quit_message_loop();
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

        /// Sans ces trois-la, la fenetre se declare non redimensionnable : un gestionnaire
        /// de fenetres en mosaique la met alors systematiquement en flottant.
        fn can_resize(&self, _window: Option<&mut Window>) -> i32 {
            1
        }

        fn can_maximize(&self, _window: Option<&mut Window>) -> i32 {
            1
        }

        fn can_minimize(&self, _window: Option<&mut Window>) -> i32 {
            1
        }

        fn initial_show_state(&self, _window: Option<&mut Window>) -> ShowState {
            self.initial_show_state
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            self.runtime_style
        }

        /// Identifie la fenetre aupres du gestionnaire de fenetres.
        ///
        /// Sans effet observable sur Wayland le 2026-09-10 : le rappel arrive avant
        /// l'affichage, le pont recopie bien les valeurs, mais la fenetre reste sans
        /// classe — Chromium les ignore pour une fenetre batie avec son systeme de vues.
        /// Le titre, lui, est fiable : c'est par lui qu'une regle de fenetrage la vise.
        fn linux_window_properties(
            &self,
            _window: Option<&mut Window>,
            properties: Option<&mut LinuxWindowProperties>,
        ) -> i32 {
            let Some(properties) = properties else { return 0 };
            properties.wayland_app_id = CefString::from(crate::flags::APP_ID);
            properties.wm_class_class = CefString::from(crate::flags::APP_ID);
            properties.wm_class_name = CefString::from(crate::flags::APP_ID);
            properties.wm_role_name = CefString::from("browser");
            1
        }
    }
}

wrap_browser_view_delegate! {
    pub struct ChromeViewDelegate {
        runtime_style: RuntimeStyle,
        is_chrome: i32,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            // `is_chrome` vaut 0 pour les vues de contenu : elles n'ont pas de taille
            // preferee, la disposition leur donne tout l'espace restant.
            let width = if self.is_chrome == 0 {
                0
            } else {
                CHROME_WIDTH_NOW.load(Ordering::Relaxed)
            };
            Size { width, height: INITIAL_HEIGHT }
        }
    }

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            self.runtime_style
        }
    }
}

/// Peint le cadre autour de la page avec la teinte de l'espace courant, pour que la
/// marge se fonde avec la barre laterale au lieu de trancher.
///
/// La marge appartient a la **fenetre**, pas au conteneur de la page : la disposition la
/// pose en retrait interieur, et le conteneur est entierement recouvert par la vue web.
/// Peindre le seul conteneur ne se voyait donc nulle part, et la gouttiere restait noire —
/// deux blocs poses cote a cote au lieu d'une fenetre. Mesure le 2026-09-10, capture a
/// l'appui.
pub fn set_accent(color: &str, host: Option<&Panel>) {
    let Some(argb) = parse_hex_color(color) else {
        debug!(%color, "teinte illisible, ignoree");
        return;
    };
    let Some(host) = host else { return };
    let view = View::from(host);
    view.set_background_color(argb);
    let Some(window) = view.window() else {
        debug!(%color, "fenetre injoignable, seule la marge interieure est peinte");
        return;
    };
    View::from(&window).set_background_color(argb);
    window.invalidate_layout();
    debug!(%color, "teinte de la fenetre et du cadre");
}

/// Lit une couleur `#rrggbb` ou `#aarrggbb` et la rend au format attendu par Chromium.
fn parse_hex_color(value: &str) -> Option<u32> {
    let hex = value.trim().strip_prefix('#')?;
    let parsed = u32::from_str_radix(hex, 16).ok()?;
    match hex.len() {
        6 => Some(0xFF00_0000 | parsed),
        8 => Some(parsed),
        _ => None,
    }
}

/// Disposition en colonnes : la barre laterale a gauche, la page a droite.
/// La marge interieure fait flotter la page au lieu de la coller aux bords.
fn side_by_side_layout() -> BoxLayoutSettings {
    BoxLayoutSettings {
        horizontal: 1,
        main_axis_alignment: AxisAlignment::START,
        cross_axis_alignment: AxisAlignment::STRETCH,
        inside_border_insets: Insets {
            top: CONTENT_INSET,
            left: 0,
            bottom: CONTENT_INSET,
            right: CONTENT_INSET,
            ..Default::default()
        },
        between_child_spacing: CONTENT_INSET,
        default_flex: 0,
        ..Default::default()
    }
}

/// Cree une vue navigateur en style Alloy, seul style compatible avec une fenetre sur mesure.
pub fn create_view(client: Option<&mut Client>, url: &str, is_chrome: i32) -> Option<BrowserView> {
    let mut delegate = ChromeViewDelegate::new(RuntimeStyle::ALLOY, is_chrome);
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
    create_view(client, url, 1)
}

// Delegue des vues posees au-dessus de la page. Leur taille est imposee par l'appelant :
// une surimpression n'a pas de place a negocier avec la disposition.
wrap_browser_view_delegate! {
    pub struct OverlayViewDelegate {
        width: i32,
        height: i32,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: self.width, height: self.height }
        }
    }

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}
