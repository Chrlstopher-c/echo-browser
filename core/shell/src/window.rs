//! Responsabilite : la fenetre du navigateur et la place respective de l'interface et du contenu.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
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

/// Teinte courante de la fenetre : c'est elle qui apparait dans les angles arrondis de la page.
static ACCENT_NOW: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(DEFAULT_SHELL);

/// Barre repliee : elle ne reserve plus de place, la page prend toute la fenetre.
static COLLAPSED: AtomicBool = AtomicBool::new(false);

/// Barre repliee mais montree par-dessus la page (la souris longe le bord gauche).
static REVEALED: AtomicBool = AtomicBool::new(false);

/// Largeur du liseré qui detecte la souris au bord gauche quand la barre est repliee.
const EDGE_WIDTH: i32 = 6;

/// La place que la barre reserve dans la disposition : aucune une fois repliee.
/// Place reservee a la barre a cet instant : elle glisse vers `target_dock` au fil des images.
static DOCK_NOW: AtomicI32 = AtomicI32::new(CHROME_WIDTH);
static DOCK_ANIMATING: AtomicBool = AtomicBool::new(false);

/// Intervalle entre deux images de l'animation d'ouverture ou de fermeture de la barre.
const DOCK_TICK_MS: i64 = 16;

pub fn docked_width() -> i32 {
    DOCK_NOW.load(Ordering::Relaxed)
}

/// La place que la barre doit finir par reserver : toute sa largeur, sauf repliee et non montree.
fn target_dock() -> i32 {
    if (COLLAPSED.load(Ordering::Relaxed) && !REVEALED.load(Ordering::Relaxed)) || floating() {
        0
    } else {
        CHROME_WIDTH_NOW.load(Ordering::Relaxed)
    }
}

/// Fenetre etroite, barre repliee mais montree : elle flotte par-dessus la page au lieu de la pousser (la page n'a
/// plus la place d'etre repoussee ; contre-audit du 09/10 : 212 px utiles).
fn floating() -> bool {
    NARROW.load(Ordering::Relaxed) && COLLAPSED.load(Ordering::Relaxed) && REVEALED.load(Ordering::Relaxed)
}

/// Lance le glissement de la barre vers sa place cible. La page suit : elle est repoussee, pas recouverte.
fn animate_dock() {
    if !DOCK_ANIMATING.swap(true, Ordering::Relaxed) {
        schedule_dock_tick();
    }
}

fn schedule_dock_tick() {
    let mut task = DockTickTask::new(());
    post_delayed_task(ThreadId::UI, Some(&mut task), DOCK_TICK_MS);
}

wrap_task! {
    struct DockTickTask {
        marker: (),
    }

    impl Task {
        fn execute(&self) {
            let target = target_dock();
            let current = DOCK_NOW.load(Ordering::Relaxed);
            let remaining = target - current;
            // Ralentit en approchant du but ; au moins un pixel par image pour ne jamais s'arreter en route.
            let step = ((remaining as f32 * 0.3).round() as i32).clamp(-remaining.abs(), remaining.abs());
            let next = if remaining.abs() <= 3 { target } else { current + if step == 0 { remaining.signum() } else { step } };
            DOCK_NOW.store(next, Ordering::Relaxed);
            if let Some(window) = SPACER.with(|slot| slot.borrow().as_ref().and_then(|p| View::from(p).window())) {
                relayout(&window);
            }
            if next == target {
                DOCK_ANIMATING.store(false, Ordering::Relaxed);
            } else {
                schedule_dock_tick();
            }
        }
    }
}

thread_local! {
    /// L'espaceur qui reserve la place de la barre : sa taille preferee est mise en cache, il faut
    /// l'invalider lui-meme pour que la disposition la relise.
    static SPACER: RefCell<Option<Panel>> = const { RefCell::new(None) };
    /// Le conteneur de la page, pour refaire la disposition (plein ecran bord a bord).
    static CONTENT_HOST: RefCell<Option<Panel>> = const { RefCell::new(None) };

    /// Le liseré de detection du bord gauche, present seulement quand la barre est repliee.
    static EDGE_STRIP: RefCell<Option<crate::overlay::Overlay>> = const { RefCell::new(None) };

    /// La barre laterale est une surimpression ancree a gauche : elle flotte au-dessus de la
    /// fenetre au lieu de pousser la page, ce qui permet de la replier sans deplacer le contenu.
    static CHROME_OVERLAY: RefCell<Option<OverlayController>> = const { RefCell::new(None) };
}

fn relayout(window: &Window) {
    SPACER.with(|slot| {
        if let Some(spacer) = slot.borrow().as_ref() {
            View::from(spacer).invalidate_layout();
        }
    });
    window.invalidate_layout();
    place_chrome(window);
}

/// Recale la surimpression de la barre sur la hauteur de la fenetre et la largeur courante.
fn place_chrome(window: &Window) {
    let size = View::from(window).bounds();
    let width = CHROME_WIDTH_NOW.load(Ordering::Relaxed);
    let collapsed = COLLAPSED.load(Ordering::Relaxed);
    let dock = DOCK_NOW.load(Ordering::Relaxed).min(width);
    let floating = floating();
    let shown = width > 0 && (dock > 0 || floating);
    EDGE_STRIP.with(|slot| {
        if let Some(strip) = slot.borrow().as_ref() {
            strip.set_bounds(Rect { x: 0, y: 0, width: EDGE_WIDTH, height: size.height });
            strip.set_visible(collapsed && !shown && !REVEALED.load(Ordering::Relaxed));
        }
    });
    CHROME_OVERLAY.with(|slot| {
        let Some(controller) = slot.borrow().clone() else { return };
        let bounds = Rect {
            // La barre glisse depuis le bord gauche : sa partie cachee sort de la fenetre.
            x: if floating { 0 } else { dock - width },
            y: CONTENT_INSET,
            width,
            height: (size.height - 2 * CONTENT_INSET).max(0),
        };
        controller.set_bounds(Some(&bounds));
        controller.set_visible(i32::from(shown));
    });
}

/// Fixe la largeur reclamee par la barre laterale et relance la disposition.
/// En dessous de cette largeur de fenetre, la barre se replie d'elle-meme : la page garde de la place.
const NARROW_BELOW: i32 = 900;
static NARROW: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn note_width(width: i32) {
    let narrow = width > 0 && width < NARROW_BELOW;
    if NARROW.swap(narrow, Ordering::Relaxed) != narrow {
        debug!(width, narrow, "fenetre etroite ou non");
        crate::bridge::publish(&echo_contract::CoreEvent::WindowNarrow { narrow });
    }
}

/// Fenetre etroite (republiee a l'interface quand elle se recharge).
pub fn is_narrow() -> bool {
    NARROW.load(Ordering::Relaxed)
}

/// Titre de la fenetre : celui de l'onglet actif, comme les autres navigateurs (barre des taches, Alt+Tab).
pub fn set_window_title(page: &str, chrome: Option<&BrowserView>) {
    let Some(window) = chrome.and_then(|c| View::from(c).window()) else { return };
    let title =
        if page.is_empty() || page.starts_with("echo://") { WINDOW_TITLE.to_string() } else { page.to_string() };
    window.set_title(Some(&CefString::from(title.as_str())));
}

/// Largeur courante de la barre.
pub fn chrome_width_now() -> i32 {
    CHROME_WIDTH_NOW.load(Ordering::Relaxed)
}

pub fn set_chrome_width(pixels: i32, chrome: Option<&BrowserView>) {
    let clamped = pixels.clamp(MIN_CHROME_WIDTH, MAX_CHROME_WIDTH);
    if CHROME_WIDTH_NOW.swap(clamped, Ordering::Relaxed) == clamped {
        return;
    }
    DOCK_NOW.store(target_dock(), Ordering::Relaxed);
    if let Some(window) = chrome.and_then(|chrome| View::from(chrome).window()) {
        relayout(&window);
    }
    debug!(largeur = clamped, "largeur de la barre laterale");
}

// Reserve a gauche la place de la barre, qui flotte par-dessus : la page commence apres elle.
/// Replie ou deplie la barre. Repliee, elle ne pousse plus la page et ne revient que par le bord gauche.
pub fn set_collapsed(collapsed: bool, chrome: Option<&BrowserView>) {
    if COLLAPSED.swap(collapsed, Ordering::Relaxed) == collapsed {
        return;
    }
    REVEALED.store(false, Ordering::Relaxed);
    let Some(chrome) = chrome else { return };
    let Some(window) = View::from(chrome).window() else { return };
    EDGE_STRIP.with(|slot| {
        let mut slot = slot.borrow_mut();
        match (collapsed, slot.is_some()) {
            (true, false) => *slot = crate::overlay::Overlay::open(chrome, EDGE_PAGE, Rect { x: 0, y: 0, width: EDGE_WIDTH, height: 1 }),
            (false, true) => {
                if let Some(strip) = slot.take() {
                    strip.close();
                }
            }
            _ => {}
        }
    });
    animate_dock();
    place_chrome(&window);
    debug!(collapsed, "barre laterale repliee ou depliee");
}

/// Montre ou cache la barre repliee, selon que la souris est au bord gauche ou la quitte.
pub fn reveal_chrome(reveal: bool, chrome: Option<&BrowserView>) {
    if !COLLAPSED.load(Ordering::Relaxed) || REVEALED.swap(reveal, Ordering::Relaxed) == reveal {
        return;
    }
    if let Some(window) = chrome.and_then(|chrome| View::from(chrome).window()) {
        place_chrome(&window);
    }
    animate_dock();
}

const EDGE_PAGE: &str = "echo://ui/bord.html";

/// Rend toutes les vues et surimpressions que ce module garde, avant l'arret de Chromium.
pub fn release_views() {
    EDGE_STRIP.with(|slot| {
        if let Some(strip) = slot.borrow_mut().take() {
            strip.close();
        }
    });
    CHROME_OVERLAY.with(|slot| {
        if let Some(controller) = slot.borrow_mut().take() {
            if controller.is_valid() == 1 {
                controller.destroy();
            }
        }
    });
    SPACER.with(|slot| *slot.borrow_mut() = None);
}

wrap_panel_delegate! {
    struct ChromeSpacerDelegate {
        marker: (),
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            // Hauteur nulle : la disposition ignore alors la vue, et la page glisse sous la barre.
            // Au moins 1 : une vue de largeur nulle sort de la disposition et garde son ancienne taille.
            Size { width: docked_width().max(1), height: 1 }
        }
    }

    impl PanelDelegate {}
}

/// Titre porte par la fenetre. Le nom du projet ne s'affiche nulle part dans l'application.
const WINDOW_TITLE: &str = "Navigateur";

/// Teinte posee avant que l'interface ne se charge : sans elle, la marge autour de la page
/// est noire pendant tout le demarrage, puis saute a la couleur de l'espace. C'est la valeur
/// plate de l'espace par defaut (`shell` de « graphite », ui/src/spaces/space-palette.ts).
const DEFAULT_SHELL: u32 = 0xFF22_2326;

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

        fn on_layout_changed(&self, view: Option<&mut View>, _new_bounds: Option<&Rect>) {
            if let Some(window) = view.and_then(|view| view.window()) {
                place_chrome(&window);
                crate::devtools::place();
                note_width(View::from(&window).bounds().width);
            }
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
            let mut spacer_delegate = ChromeSpacerDelegate::new(());
            if let Some(spacer) = panel_create(Some(&mut spacer_delegate)) {
                window.add_child_view(Some(&mut View::from(&spacer)));
                SPACER.with(|slot| *slot.borrow_mut() = Some(spacer));
            }
            let mut chrome_view = View::from(chrome);
            let controller = window.add_overlay_view(Some(&mut chrome_view), DockingMode::CUSTOM, 1);
            CHROME_OVERLAY.with(|slot| *slot.borrow_mut() = controller);

            let host = self.content_host.borrow().clone();
            CONTENT_HOST.with(|slot| *slot.borrow_mut() = host.clone());
            if let Some(host) = host.as_ref() {
                let mut host_view = View::from(host);
                window.add_child_view(Some(&mut host_view));
                if let Some(layout) = layout {
                    layout.set_flex_for_view(Some(&mut host_view), 1);
                }
            }

            View::from(&*window).set_background_color(ACCENT_NOW.load(Ordering::Relaxed));
            window.set_title(Some(&CefString::from(WINDOW_TITLE)));
            place_chrome(window);
            window.show();
            info!(contenu = host.is_some(), "fenetre du navigateur affichee");
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            crate::persist::flush();
            *self.chrome_view.borrow_mut() = None;
            *self.content_host.borrow_mut() = None;
            info!("fenetre fermee, arret du navigateur");
            crate::anchor::quit();
        }

        /// Toute demande de fermeture de la fenetre (raccourci du gestionnaire de fenetres, bouton, signal) ferme
        /// l'application entiere, quelle que soit la vue qui avait le focus. Fermer seulement la vue de l'interface
        /// la laissait grise et la fenetre ouverte. Les onglets sont enregistres avant.
        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            CLOSING.store(true, Ordering::Relaxed);
            crate::persist::flush();
            1
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

        /// Une page ouvre une fenetre (`window.open`, connexion Google…) : elle devient un onglet d'Echo au
        /// lieu d'une fenetre a part. Le lien avec la page d'origine (`window.opener`) est garde : la
        /// connexion revient bien sur l'onglet qui l'a demandee. Les DevTools restent une fenetre.
        /// La popup recoit le meme delegue : ses propres popups (fenetre ouverte depuis une fenetre ouverte)
        /// deviennent elles aussi des onglets, au lieu de fenetres a part.
        fn delegate_for_popup_browser_view(
            &self,
            _browser_view: Option<&mut BrowserView>,
            _settings: Option<&BrowserSettings>,
            _client: Option<&mut Client>,
            is_devtools: i32,
        ) -> Option<BrowserViewDelegate> {
            if is_devtools == 1 || self.is_chrome == 1 {
                return None;
            }
            Some(ChromeViewDelegate::new(RuntimeStyle::ALLOY, 0))
        }

        fn on_popup_browser_view_created(
            &self,
            _browser_view: Option<&mut BrowserView>,
            popup_browser_view: Option<&mut BrowserView>,
            is_devtools: i32,
        ) -> i32 {
            let Some(popup) = popup_browser_view else { return 0 };
            if is_devtools == 1 || self.is_chrome == 1 {
                return 0;
            }
            i32::from(crate::bridge::adopt_popup(popup.clone()))
        }
    }
}

/// La fenetre se ferme : chaque navigateur doit alors se fermer normalement.
pub static CLOSING: AtomicBool = AtomicBool::new(false);

/// Peint le cadre autour de la page avec la teinte de l'espace courant, pour que la
/// marge se fonde avec la barre laterale au lieu de trancher.
///
/// La marge appartient a la **fenetre**, pas au conteneur de la page : la disposition la
/// pose en retrait interieur, et le conteneur est entierement recouvert par la vue web.
/// Peindre le seul conteneur ne se voyait donc nulle part, et la gouttiere restait noire —
/// deux blocs poses cote a cote au lieu d'une fenetre. Mesure le 2026-09-10, capture a
/// l'appui.
/// Reprend la teinte de la derniere session, avant que la fenetre n'existe.
pub fn restore_accent(color: &str) {
    if let Some(argb) = parse_hex_color(color) {
        ACCENT_NOW.store(argb, Ordering::Relaxed);
    }
}

/// La teinte courante de la fenetre, en ARGB.
pub fn accent_now() -> u32 {
    ACCENT_NOW.load(Ordering::Relaxed)
}

pub fn set_accent(color: &str, host: Option<&Panel>) {
    let Some(argb) = parse_hex_color(color) else {
        debug!(%color, "teinte illisible, ignoree");
        return;
    };
    ACCENT_NOW.store(argb, Ordering::Relaxed);
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

/// Plein ecran : la page va jusqu'aux bords (ni marge ni teinte autour) ; sinon elle reflotte.
pub fn set_edge_to_edge(on: bool, chrome: Option<&BrowserView>) {
    let Some(window) = chrome.and_then(|c| View::from(c).window()) else { return };
    // Comme Chrome : la video prend l'ecran entier, pas seulement la fenetre.
    window.set_fullscreen(i32::from(on));
    let layout = window.set_to_box_layout(Some(&layout_with_inset(if on { 0 } else { CONTENT_INSET })));
    CONTENT_HOST.with(|slot| {
        if let (Some(layout), Some(host)) = (layout, slot.borrow().as_ref()) {
            layout.set_flex_for_view(Some(&mut View::from(host)), 1);
        }
    });
    relayout(&window);
}

/// Disposition en colonnes : la barre laterale a gauche, la page a droite.
/// La marge interieure fait flotter la page au lieu de la coller aux bords.
fn side_by_side_layout() -> BoxLayoutSettings {
    layout_with_inset(CONTENT_INSET)
}

fn layout_with_inset(inset: i32) -> BoxLayoutSettings {
    BoxLayoutSettings {
        horizontal: 1,
        main_axis_alignment: AxisAlignment::START,
        cross_axis_alignment: AxisAlignment::STRETCH,
        inside_border_insets: Insets {
            top: inset,
            left: 0,
            bottom: inset,
            right: inset,
            ..Default::default()
        },
        between_child_spacing: inset,
        default_flex: 0,
        ..Default::default()
    }
}

/// Cree une vue navigateur en style Alloy, seul style compatible avec une fenetre sur mesure.
pub fn create_view(
    client: Option<&mut Client>,
    url: &str,
    is_chrome: i32,
    container: Option<&str>,
) -> Option<BrowserView> {
    let mut context = container.and_then(crate::containers::context_for);
    let mut delegate = ChromeViewDelegate::new(RuntimeStyle::ALLOY, is_chrome);
    browser_view_create(
        client,
        Some(&CefString::from(url)),
        Some(&BrowserSettings::default()),
        None,
        context.as_mut(),
        Some(&mut delegate),
    )
}

/// Cree la vue qui porte l'interface du navigateur.
pub fn create_chrome_view(client: Option<&mut Client>, url: &str) -> Option<BrowserView> {
    // Style Alloy impose : une vue en style Chrome ajoutee a une fenetre sur mesure cherche
    // l'infrastructure d'onglets du vrai Chrome et fait planter le processus dans
    // tabs::TabInterface::GetFromContents. Mesure le 2026-09-10, pile a l'appui.
    create_view(client, url, 1, None)
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
