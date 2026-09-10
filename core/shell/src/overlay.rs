//! Responsabilite : les vues posees **au-dessus** de la page.
//!
//! La vue web est une surface native opaque : rien de l'interface ne pouvait s'afficher
//! par-dessus, ce qui interdisait a la fois le menu contextuel, la palette du nouvel
//! onglet, le rail flottant et les fenetres d'extension. `Window::add_overlay_view` leve
//! le mur : une vue ancree librement au-dessus de la page, dont on pilote la position,
//! la taille et la visibilite.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee.
use cef::*;
use tracing::{debug, warn};

/// Une vue superposee vivante et son controleur.
pub struct Overlay {
    controller: OverlayController,
    view: BrowserView,
}

impl Overlay {
    /// Ouvre une vue superposee chargee sur `url`, posee aux coordonnees demandees.
    ///
    /// `anchor` est n'importe quelle vue deja dans la fenetre : elle sert uniquement a
    /// retrouver la fenetre, qui seule sait ancrer une surimpression.
    pub fn open(anchor: &BrowserView, url: &str, bounds: Rect) -> Option<Self> {
        let window = View::from(anchor).window()?;
        // Fond transparent : la page reste visible sous les coins et les ombres.
        let settings = BrowserSettings { background_color: 0, ..Default::default() };
        let mut delegate = crate::window::OverlayViewDelegate::new(bounds.width, bounds.height);
        // Le client partage porte les memes rappels que les onglets : sans lui, la vue
        // n'a ni gestionnaire de requete, ni de chargement, ni de duree de vie.
        let mut client = crate::session::with(|s| s.client.clone()).flatten();
        let view = browser_view_create(
            client.as_mut(),
            Some(&CefString::from(url)),
            Some(&settings),
            None,
            None,
            Some(&mut delegate),
        )?;
        let mut as_view = View::from(&view);
        let controller = window.add_overlay_view(Some(&mut as_view), DockingMode::CUSTOM, 1)?;
        controller.set_bounds(Some(&bounds));
        controller.set_visible(1);
        debug!(%url, x = bounds.x, y = bounds.y, "surimpression ouverte");
        Some(Self { controller, view })
    }

    /// Deplace et redimensionne la surimpression.
    pub fn set_bounds(&self, bounds: Rect) {
        self.controller.set_bounds(Some(&bounds));
    }

    /// La surimpression est-elle reellement dessinee ? Sert de controle, pas de decor.
    pub fn is_drawn(&self) -> bool {
        self.controller.is_valid() == 1 && self.controller.is_drawn() == 1
    }

    /// Cadre de la vue dans la fenetre, tel que Chromium le voit apres disposition.
    pub fn bounds(&self) -> Rect {
        self.controller.bounds()
    }

    /// La trame de la page portee, pour lui parler.
    pub fn frame(&self) -> Option<Frame> {
        self.view.browser()?.main_frame()
    }

    /// Retire la surimpression. Sans cet appel, la vue survit a l'objet.
    pub fn close(self) {
        if self.controller.is_valid() == 1 {
            self.controller.destroy();
        } else {
            warn!("surimpression deja invalide a la fermeture");
        }
    }
}

thread_local! {
    /// La variante « soeur » garde sa vue vivante.
    static TEST_SIBLING: std::cell::RefCell<Option<BrowserView>> =
        const { std::cell::RefCell::new(None) };

    /// La variante « panneau » garde son controleur vivant, pour la meme raison.
    static TEST_PANEL: std::cell::RefCell<Option<OverlayController>> =
        const { std::cell::RefCell::new(None) };

    /// L'essai garde sa vue vivante : laisser tomber l'objet retirerait la surimpression
    /// aussitot posee, et l'essai ne prouverait rien.
    static TEST_OVERLAY: std::cell::RefCell<Option<Overlay>> =
        const { std::cell::RefCell::new(None) };
}

/// Essai de faisabilite, arme par `ECHO_OVERLAY_TEST=1`.
///
/// Tout le reste — menu contextuel, palette, rail flottant, fenetres d'extension — repose
/// sur la surimpression. Avant de batir dessus, on verifie qu'une vue transparente
/// s'affiche reellement au-dessus de la page, et pas seulement qu'elle se declare valide.
pub fn arm_test() {
    if std::env::var_os("ECHO_OVERLAY_TEST").is_none() {
        return;
    }
    tracing::info!("essai de surimpression arme");
    let mut task = OverlayTestTask::new(0);
    post_delayed_task(ThreadId::UI, Some(&mut task), 7_000);
}

const TEST_PAGE: &str = "echo://ui/essai-surimpression.html";

wrap_task! {
    struct OverlayTestTask {
        _step: i32,
    }

    impl Task {
        fn execute(&self) {
            let anchor = crate::session::with(|s| s.chrome.clone()).flatten();
            let Some(anchor) = anchor else {
                warn!("essai de surimpression : pas de vue d'ancrage");
                return;
            };
            let bounds = Rect { x: 420, y: 260, width: 320, height: 184 };
            // Variante « extension » : ouvre la fenetre de la premiere extension qui en
            // declare une. Verifie le point incertain — une page chrome-extension:// se
            // charge-t-elle dans une vue posee au-dessus de la page ?
            if std::env::var_os("ECHO_OVERLAY_TEST").is_some_and(|v| v == "extension") {
                let vise = std::env::var("ECHO_OVERLAY_EXT").ok();
                let cible = crate::session::with(|s| {
                    s.extensions
                        .list()
                        .into_iter()
                        .find(|e| {
                            e.action.popup.is_some()
                                && e.enabled
                                && vise.as_deref().is_none_or(|id| id == e.id)
                        })
                        .map(|e| (e.id.clone(), e.action.popup.clone().unwrap_or_default()))
                })
                .flatten();
                let Some((id, chemin)) = cible else {
                    warn!("essai extension : aucune extension avec fenetre");
                    return;
                };
                let url = echo_extensions::action::resource_url(&id, &chemin);
                // Voie « onglet » : la meme adresse, ouverte comme une page ordinaire.
                // Si elle passe la, l'obstacle tient a la vue superposee, pas au
                // schema ; si elle echoue aussi, c'est Chromium qui refuse la page
                // d'extension a une vue non privilegiee.
                if std::env::var_os("ECHO_OVERLAY_ONGLET").is_some() {
                    tracing::info!(%url, "essai extension : ouverture en onglet");
                    crate::bridge::open_tab(&url);
                    return;
                }
                let ancre = Rect { x: 60, y: 120, width: 28, height: 28 };
                toggle_extension_popup(&id, &url, ancre, &anchor);
                tracing::info!(%id, %url, ouverte = open_popup_id().is_some(),
                    "essai extension : fenetre demandee");
                return;
            }

            // Variante « soeur » : une seconde vue web ajoutee au meme conteneur que la
            // page, apres elle. Si les surfaces natives se composent dans l'ordre des
            // enfants, c'est la voie ; sinon, aucune surimpression n'est possible.
            if std::env::var_os("ECHO_OVERLAY_TEST").is_some_and(|v| v == "soeur") {
                let host = crate::session::with(|s| s.tabs.host()).flatten();
                let Some(host) = host else { return };
                let mut delegate = crate::window::OverlayViewDelegate::new(320, 184);
                let settings = BrowserSettings { background_color: 0, ..Default::default() };
                let view = browser_view_create(
                    None,
                    Some(&CefString::from(TEST_PAGE)),
                    Some(&settings),
                    None,
                    None,
                    Some(&mut delegate),
                );
                let Some(view) = view else {
                    warn!("essai soeur : creation refusee");
                    return;
                };
                let mut as_view = View::from(&view);
                host.add_child_view(Some(&mut as_view));
                tracing::info!(dessinee = as_view.is_drawn() == 1, "essai soeur : vue ajoutee");
                // Controle de reference : si la page d'essai ne se charge pas, les trois
                // essais ne prouvent rien. On masque la vue de l'onglet — l'essai doit
                // alors apparaitre seul.
                if std::env::var_os("ECHO_OVERLAY_HIDE_PAGE").is_some() {
                    let active = crate::session::with(|s| s.tabs.active().map(|t| t.view.clone())).flatten();
                    if let Some(active) = active {
                        View::from(&active).set_visible(0);
                        tracing::info!("controle : vue de l'onglet masquee");
                    }
                }
                TEST_SIBLING.with(|cell| *cell.borrow_mut() = Some(view));
                return;
            }

            // Variante « panneau » : une vue native opaque, sans page web. Elle isole la
            // question de fond — une surimpression peut-elle seulement s'afficher au-dessus
            // d'une vue web, qui est une surface native sur Linux ?
            if std::env::var_os("ECHO_OVERLAY_TEST").is_some_and(|v| v == "panel") {
                let Some(window) = View::from(&anchor).window() else { return };
                let Some(panel) = panel_create(None) else {
                    warn!("essai panneau : creation refusee");
                    return;
                };
                View::from(&panel).set_background_color(0xFFDC_5A3C);
                let mut as_view = View::from(&panel);
                let Some(controller) =
                    window.add_overlay_view(Some(&mut as_view), DockingMode::CUSTOM, 0)
                else {
                    warn!("essai panneau : ancrage refuse");
                    return;
                };
                controller.set_bounds(Some(&bounds));
                controller.set_visible(1);
                let posee = controller.bounds();
                tracing::info!(
                    dessine = controller.is_drawn() == 1,
                    x = posee.x, y = posee.y, l = posee.width, h = posee.height,
                    "essai panneau : vue native posee"
                );
                TEST_PANEL.with(|cell| *cell.borrow_mut() = Some(controller));
                return;
            }
            let Some(overlay) = Overlay::open(&anchor, TEST_PAGE, bounds) else {
                warn!("essai de surimpression : ouverture refusee");
                return;
            };
            let posee = overlay.bounds();
            tracing::info!(
                dessinee = overlay.is_drawn(),
                x = posee.x, y = posee.y, l = posee.width, h = posee.height,
                "essai de surimpression : vue posee"
            );
            TEST_OVERLAY.with(|cell| *cell.borrow_mut() = Some(overlay));
        }
    }
}

/// Dimensions de depart d'une fenetre d'extension. Chrome mesure la page pour s'y
/// ajuster ; faute de pouvoir l'interroger, on prend la taille la plus courante et on
/// laisse la page defiler dedans.
const POPUP_WIDTH: i32 = 380;
const POPUP_HEIGHT: i32 = 600;

/// Ecart entre l'icone et la fenetre qu'elle ouvre.
const POPUP_GAP: i32 = 6;

/// Marge minimale entre la fenetre d'extension et le bord de la fenetre du navigateur.
const POPUP_MARGIN: i32 = 8;

thread_local! {
    /// La fenetre d'extension ouverte, s'il y en a une. Une seule a la fois : c'est ce
    /// que fait Chrome, et deux fenetres ouvertes n'auraient pas de sens a l'usage.
    static POPUP: std::cell::RefCell<Option<(String, Overlay)>> =
        const { std::cell::RefCell::new(None) };
}

/// Identifiant de l'extension dont la fenetre est ouverte.
pub fn open_popup_id() -> Option<String> {
    POPUP.with(|cell| cell.borrow().as_ref().map(|(id, _)| id.clone()))
}

/// Ouvre la fenetre d'une extension sous son icone. Rouvrir la meme la referme :
/// c'est le comportement attendu d'un bouton a bascule.
pub fn toggle_extension_popup(id: &str, url: &str, anchor: Rect, anchor_view: &BrowserView) {
    if open_popup_id().as_deref() == Some(id) {
        close_extension_popup();
        return;
    }
    close_extension_popup();
    let bounds = place_under(anchor, anchor_view);
    let (x, y) = (bounds.x, bounds.y);
    let Some(overlay) = Overlay::open(anchor_view, url, bounds) else {
        warn!(%id, "fenetre d'extension : ouverture refusee");
        return;
    };
    debug!(%id, %url, x, y, "fenetre d'extension ouverte");
    POPUP.with(|cell| *cell.borrow_mut() = Some((id.to_string(), overlay)));
}

/// Referme la fenetre d'extension ouverte. Sans effet s'il n'y en a pas.
pub fn close_extension_popup() {
    let previous = POPUP.with(|cell| cell.borrow_mut().take());
    if let Some((id, overlay)) = previous {
        overlay.close();
        debug!(%id, "fenetre d'extension fermee");
    }
}

/// Pose la fenetre sous son ancre, sans deborder de la fenetre du navigateur.
///
/// L'interface donne l'ancre dans ses propres coordonnees ; la surimpression, elle, se
/// place dans celles de la fenetre. La vue de l'interface n'etant pas collee au coin —
/// la disposition lui pose une marge — les deux reperes different, et l'ecart se lit sur
/// la vue elle-meme plutot que de se deviner.
fn place_under(anchor: Rect, anchor_view: &BrowserView) -> Rect {
    let view = View::from(anchor_view);
    let frame = view
        .window()
        .map(|window| View::from(&window).bounds())
        .unwrap_or(Rect { x: 0, y: 0, width: 1440, height: 900 });
    let origine = view.bounds();
    let anchor = Rect {
        x: anchor.x + origine.x,
        y: anchor.y + origine.y,
        width: anchor.width,
        height: anchor.height,
    };

    let height = POPUP_HEIGHT.min(frame.height - 2 * POPUP_MARGIN).max(200);
    let width = POPUP_WIDTH.min(frame.width - 2 * POPUP_MARGIN).max(240);
    let x = anchor.x.min(frame.width - width - POPUP_MARGIN).max(POPUP_MARGIN);
    let y = (anchor.y + anchor.height + POPUP_GAP)
        .min(frame.height - height - POPUP_MARGIN)
        .max(POPUP_MARGIN);
    Rect { x, y, width, height }
}
