//! Responsabilite : les onglets — creation, bascule, fermeture, et la vue qui leur sert de scene.
//!
//! Toutes les vues d'onglets vivent dans le meme conteneur et se superposent ; seule
//! celle de l'onglet actif est visible. Basculer ne recharge donc jamais une page.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use echo_contract::{TabId, TabView};
use tracing::{debug, warn};

/// Un onglet et la vue qui l'affiche.
pub struct Tab {
    pub id: TabId,
    pub view: BrowserView,
    pub title: String,
    pub url: String,
    pub loading: bool,
}

impl Tab {
    fn to_view(&self) -> TabView {
        let (can_go_back, can_go_forward) = self
            .view
            .browser()
            .map(|browser| (browser.can_go_back() == 1, browser.can_go_forward() == 1))
            .unwrap_or((false, false));
        TabView {
            id: self.id,
            title: self.title.clone(),
            url: self.url.clone(),
            loading: self.loading,
            progress: if self.loading { 0.4 } else { 1.0 },
            can_go_back,
            can_go_forward,
            favicon: None,
        }
    }

    /// Vrai si cette vue porte le navigateur donne.
    pub fn owns(&self, browser_id: i32) -> bool {
        self.view.browser().map(|b| b.identifier()) == Some(browser_id)
    }
}

/// Ce qu'il reste a faire cote Chromium apres avoir retire un onglet de la liste.
pub struct Detached {
    pub view: Option<BrowserView>,
    pub host: Option<Panel>,
    /// Nombre d'onglets encore ouverts.
    pub remaining: usize,
}

impl Detached {
    /// Retire la vue de la scene. A appeler hors de l'acces a l'etat.
    ///
    /// On ne ferme **pas** le navigateur explicitement : pour une vue posee dans une
    /// fenetre sur mesure, cette demande remonte jusqu'a la fenetre et la ferme
    /// entierement — mesure le 2026-09-10, fermer un onglet fermait le navigateur.
    /// Retirer la vue suffit : Chromium libere le navigateur avec elle.
    pub fn dispose(self) {
        let (Some(view), Some(host)) = (self.view, self.host) else { return };
        host.remove_child_view(Some(&mut View::from(&view)));
    }
}

/// L'ensemble des onglets ouverts et leur scene commune.
#[derive(Default)]
pub struct Tabs {
    host: Option<Panel>,
    entries: Vec<Tab>,
    active: Option<TabId>,
    next_id: TabId,
}

impl Tabs {
    /// Le conteneur des vues d'onglets, cree au premier appel.
    pub fn host(&mut self) -> Option<Panel> {
        if self.host.is_none() {
            let panel = panel_create(None);
            if let Some(panel) = panel.as_ref() {
                panel.set_to_fill_layout();
            }
            self.host = panel;
        }
        self.host.clone()
    }

    /// Enregistre une vue deja creee et rattachee, et la rend active.
    ///
    /// La creation de la vue et son rattachement se font **hors** de l'acces a l'etat :
    /// Chromium rappelle le programme pendant ces appels.
    pub fn adopt(&mut self, view: BrowserView, url: &str) -> TabId {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(Tab {
            id,
            view,
            title: url.to_string(),
            url: url.to_string(),
            loading: true,
        });
        self.select(id);
        debug!(id, %url, "onglet ouvert");
        id
    }

    /// Rend un onglet visible et masque les autres.
    pub fn select(&mut self, id: TabId) {
        if !self.entries.iter().any(|tab| tab.id == id) {
            warn!(id, "onglet inconnu");
            return;
        }
        self.active = Some(id);
        for tab in &self.entries {
            View::from(&tab.view).set_visible(i32::from(tab.id == id));
        }
    }

    /// Retire un onglet de la liste et renvoie sa vue, sans toucher a Chromium.
    ///
    /// Le detachement et la fermeture se font **hors** de l'acces a l'etat : ces appels
    /// rendent la main a Chromium, qui rappelle le programme au milieu.
    pub fn detach(&mut self, id: TabId) -> Detached {
        let Some(index) = self.entries.iter().position(|tab| tab.id == id) else {
            return Detached { view: None, host: self.host.clone(), remaining: self.entries.len() };
        };
        let tab = self.entries.remove(index);
        if self.active == Some(id) {
            self.active = self.entries.get(index).or_else(|| self.entries.last()).map(|t| t.id);
        }
        Detached { view: Some(tab.view), host: self.host.clone(), remaining: self.entries.len() }
    }

    /// Rend visible l'onglet actif. A appeler apres un detachement.
    pub fn refresh_visibility(&self) {
        for tab in &self.entries {
            View::from(&tab.view).set_visible(i32::from(Some(tab.id) == self.active));
        }
    }

    pub fn active(&self) -> Option<&Tab> {
        let id = self.active?;
        self.entries.iter().find(|tab| tab.id == id)
    }

    pub fn active_id(&self) -> Option<TabId> {
        self.active
    }

    /// Retrouve l'onglet qui porte un navigateur donne.
    pub fn by_browser(&mut self, browser_id: i32) -> Option<&mut Tab> {
        self.entries.iter_mut().find(|tab| tab.owns(browser_id))
    }

    pub fn get_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        self.entries.iter_mut().find(|tab| tab.id == id)
    }

    /// L'etat de tous les onglets, dans l'ordre d'affichage.
    pub fn snapshot(&self) -> Vec<TabView> {
        self.entries.iter().map(Tab::to_view).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
