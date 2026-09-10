//! Responsabilite : l'etat vivant du navigateur — les onglets, l'interface, le bouclier.
//!
//! Tout cet etat n'existe que sur le thread interface de Chromium : les objets CEF ne
//! traversent pas les threads. Les autres threads passent par la file du pont.

use crate::tabs::Tabs;
use cef::{BrowserView, CefString, Client, Frame, ImplBrowser, ImplBrowserView, ImplFrame};
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
    /// Le client partage par toutes les vues, necessaire pour ouvrir un onglet.
    pub client: Option<Client>,
    pub tabs: Tabs,
    pub shield: Arc<Shield>,
}

impl Session {
    /// La frame de l'interface, cible des evenements pousses. Elle n'existe qu'une fois
    /// la vue rattachee a la fenetre.
    pub fn chrome_frame(&self) -> Option<Frame> {
        self.chrome.as_ref()?.browser()?.main_frame()
    }

    /// La frame de la page affichee dans l'onglet actif.
    pub fn active_frame(&self) -> Option<Frame> {
        self.tabs.active()?.view.browser()?.main_frame()
    }

    /// L'adresse affichee dans l'onglet actif.
    pub fn active_url(&self) -> String {
        self.active_frame()
            .map(|frame| CefString::from(&frame.url()).to_string())
            .unwrap_or_default()
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
