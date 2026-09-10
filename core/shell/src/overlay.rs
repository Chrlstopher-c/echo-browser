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
        let view = browser_view_create(
            None,
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
