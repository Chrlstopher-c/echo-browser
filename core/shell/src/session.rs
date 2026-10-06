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
    pub extensions: echo_extensions::Extensions,
    pub library: std::sync::Arc<echo_library::Library>,
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
        self.tabs.active()?.browser()?.main_frame()
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

/// Donne acces a la session sur le thread interface. Renvoie `None` ailleurs, ou si
/// l'etat est deja en cours de modification.
///
/// L'acces est tentant, jamais force : Chromium rappelle le programme au milieu de ses
/// propres appels (une navigation declenchee ici revient signaler qu'elle charge). Un
/// acces force ferait paniquer le processus, donc mourir le navigateur — mesure le
/// 2026-09-10 en ouvrant un onglet. A l'appelant de ne pas tenir l'acces pendant un
/// appel a Chromium.
pub fn with<R>(f: impl FnOnce(&mut Session) -> R) -> Option<R> {
    SESSION.with(|cell| match cell.try_borrow_mut() {
        Ok(mut guard) => guard.as_mut().map(f),
        Err(_) => {
            tracing::warn!("etat du navigateur deja en cours de modification, acces ignore");
            None
        }
    })
}
