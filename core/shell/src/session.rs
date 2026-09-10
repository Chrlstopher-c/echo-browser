//! Responsabilite : l'etat vivant du navigateur — les vues, la frame d'interface, le bouclier.
//!
//! Tout cet etat n'existe que sur le thread interface de Chromium : les objets CEF ne
//! traversent pas les threads. Les autres threads passent par la file du pont.

use cef::{BrowserView, Frame, ImplBrowser, ImplBrowserView};
use echo_shield::Shield;
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
}

/// Les pieces du navigateur manipulees par le pont.
pub struct Session {
    /// La vue qui porte l'interface.
    pub chrome: Option<BrowserView>,
    /// La vue qui affiche la page web.
    pub content: Option<BrowserView>,
    pub shield: Arc<Shield>,
}

impl Session {
    /// La frame de l'interface, cible des evenements pousses. Elle n'existe qu'une fois
    /// la vue rattachee a la fenetre.
    pub fn chrome_frame(&self) -> Option<Frame> {
        self.chrome.as_ref()?.browser()?.main_frame()
    }

    /// La frame de la page affichee.
    pub fn content_frame(&self) -> Option<Frame> {
        self.content.as_ref()?.browser()?.main_frame()
    }
}

/// Installe la session sur le thread courant. A appeler une seule fois, au demarrage.
pub fn install(session: Session) {
    SESSION.with(|cell| *cell.borrow_mut() = Some(session));
}

/// Donne acces a la session sur le thread interface. Renvoie `None` ailleurs.
pub fn with<R>(f: impl FnOnce(&mut Session) -> R) -> Option<R> {
    SESSION.with(|cell| cell.borrow_mut().as_mut().map(f))
}
