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

    /// Ouvre un onglet et le rend actif.
    pub fn open(&mut self, client: Option<&mut Client>, url: &str) -> Option<TabId> {
        let view = crate::window::create_view(client, url, 0)?;
        let host = self.host()?;
        let mut child = View::from(&view);
        host.add_child_view(Some(&mut child));

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
        Some(id)
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

    /// Ferme un onglet. Renvoie vrai s'il ne reste plus rien d'ouvert.
    pub fn close(&mut self, id: TabId) -> bool {
        let Some(index) = self.entries.iter().position(|tab| tab.id == id) else {
            return self.entries.is_empty();
        };
        let tab = self.entries.remove(index);
        if let Some(browser) = tab.view.browser().and_then(|b| b.host()) {
            browser.close_browser(1);
        }
        if self.active == Some(id) {
            let fallback = self.entries.get(index).or_else(|| self.entries.last());
            match fallback.map(|tab| tab.id) {
                Some(next) => self.select(next),
                None => self.active = None,
            }
        }
        self.entries.is_empty()
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
