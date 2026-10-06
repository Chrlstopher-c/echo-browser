//! Responsabilite : les onglets — creation, bascule, fermeture, et la vue qui leur sert de scene.
//!
//! Toutes les vues d'onglets vivent dans le meme conteneur et se superposent ; seule
//! celle de l'onglet actif est visible. Basculer ne recharge donc jamais une page.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use echo_contract::{Security, TabId, TabView};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Un onglet et la vue qui l'affiche.
pub struct Tab {
    pub id: TabId,
    /// `None` quand l'onglet dort : le navigateur est detruit, seule la fiche reste.
    pub view: Option<BrowserView>,
    pub asleep: bool,
    /// Vrai quand l'utilisateur a saisi quelque chose dans la page : l'endormir perdrait sa saisie.
    pub dirty: bool,
    /// Dernier moment ou l'onglet a ete l'onglet actif.
    pub last_active: Instant,
    pub title: String,
    pub url: String,
    pub loading: bool,
    /// Fil des pages visitees dans cet onglet, du plus ancien au plus recent.
    ///
    /// Chromium tient le sien, mais ne permet pas de le rendre a un onglet neuf : apres
    /// une relance, le bouton precedent repartirait de zero. Celui-ci survit, au prix
    /// d'un rechargement de la page au lieu d'un retour instantane.
    pub history: Vec<String>,
    /// Position courante dans `history`.
    pub position: usize,
    pub pinned: bool,
    /// Facteur de zoom, 1.0 etant la taille naturelle.
    pub zoom: f32,
    /// Vrai quand la page joue du son.
    pub audible: bool,
}

impl Tab {
    pub fn browser(&self) -> Option<Browser> {
        self.view.as_ref()?.browser()
    }

    /// Enregistre une page atteinte. Ignore un rechargement de la meme adresse.
    pub fn record_visit(&mut self, url: &str) {
        if url.is_empty() || self.history.get(self.position).map(String::as_str) == Some(url) {
            return;
        }
        self.history.truncate(self.position + 1);
        self.history.push(url.to_string());
        self.position = self.history.len() - 1;
    }

    /// Adresse precedente dans notre fil, si Chromium ne peut pas y retourner lui-meme.
    pub fn previous_url(&self) -> Option<&str> {
        self.position.checked_sub(1).and_then(|index| self.history.get(index)).map(String::as_str)
    }

    /// Adresse suivante dans notre fil.
    pub fn next_url(&self) -> Option<&str> {
        self.history.get(self.position + 1).map(String::as_str)
    }

    pub fn step(&mut self, forward: bool) {
        if forward {
            self.position = (self.position + 1).min(self.history.len().saturating_sub(1));
        } else {
            self.position = self.position.saturating_sub(1);
        }
    }
}

impl Tab {
    fn to_view(&self) -> TabView {
        let (native_back, native_forward) = self
            .browser()
            .map(|browser| (browser.can_go_back() == 1, browser.can_go_forward() == 1))
            .unwrap_or((false, false));
        let can_go_back = native_back || self.previous_url().is_some();
        let can_go_forward = native_forward || self.next_url().is_some();
        TabView {
            id: self.id,
            title: self.title.clone(),
            url: self.url.clone(),
            loading: self.loading,
            progress: if self.loading { 0.4 } else { 1.0 },
            can_go_back,
            can_go_forward,
            favicon: None,
            security: security_of(&self.url),
            pinned: self.pinned,
            zoom: self.zoom,
            audible: self.audible,
            asleep: self.asleep,
        }
    }

    /// Vrai si cette vue porte le navigateur donne.
    pub fn owns(&self, browser_id: i32) -> bool {
        self.browser().map(|b| b.identifier()) == Some(browser_id)
    }
}

/// Etat de la connexion, lu depuis l'adresse. Le detail du certificat viendra plus tard.
fn security_of(url: &str) -> Security {
    if url.starts_with("https://") {
        Security::Secure
    } else if url.starts_with("http://") {
        Security::Insecure
    } else {
        Security::Local
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
            view: Some(view),
            asleep: false,
            dirty: false,
            last_active: Instant::now(),
            title: url.to_string(),
            url: url.to_string(),
            loading: true,
            history: vec![url.to_string()],
            position: 0,
            pinned: false,
            zoom: 1.0,
            audible: false,
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
        if let Some(tab) = self.get_mut(id) {
            tab.last_active = Instant::now();
        }
        self.refresh_visibility();
    }

    pub fn exists(&self, id: TabId) -> bool {
        self.entries.iter().any(|tab| tab.id == id)
    }

    pub fn find_by_url(&self, url: &str) -> Option<TabId> {
        self.entries.iter().find(|tab| tab.url == url).map(|tab| tab.id)
    }

    pub fn is_asleep(&self, id: TabId) -> bool {
        self.entries.iter().any(|tab| tab.id == id && tab.asleep)
    }

    /// Detruit le navigateur d'un onglet inactif pour rendre sa memoire, en gardant la fiche.
    ///
    /// Refuse l'onglet actif, celui qui joue du son et celui qui charge. Le detachement se
    /// fait hors de l'acces a l'etat, comme pour une fermeture.
    pub fn put_to_sleep(&mut self, id: TabId) -> Option<Detached> {
        if self.active == Some(id) {
            return None;
        }
        let host = self.host.clone();
        let tab = self.entries.iter_mut().find(|tab| tab.id == id)?;
        if tab.audible || tab.loading || tab.asleep || tab.dirty {
            return None;
        }
        let view = tab.view.take()?;
        tab.asleep = true;
        debug!(id, url = %tab.url, "onglet endormi");
        Some(Detached { view: Some(view), host, remaining: self.entries.len() })
    }

    /// Les onglets inactifs depuis au moins `idle`, bons a endormir.
    pub fn sleep_candidates(&self, idle: Duration) -> Vec<TabId> {
        self.entries
            .iter()
            .filter(|tab| {
                !tab.asleep
                    && Some(tab.id) != self.active
                    && !tab.audible
                    && !tab.dirty
                    && !tab.loading
                    && tab.last_active.elapsed() >= idle
            })
            .map(|tab| tab.id)
            .collect()
    }

    /// L'adresse a recharger au reveil d'un onglet.
    pub fn wake_url(&self, id: TabId) -> Option<String> {
        self.entries.iter().find(|tab| tab.id == id).map(|tab| tab.url.clone())
    }

    /// Rend une vue neuve a un onglet endormi.
    pub fn wake_with(&mut self, id: TabId, view: BrowserView) {
        let Some(tab) = self.get_mut(id) else { return };
        tab.view = Some(view);
        tab.asleep = false;
        tab.loading = true;
        debug!(id, url = %tab.url, "onglet reveille");
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
        Detached { view: tab.view, host: self.host.clone(), remaining: self.entries.len() }
    }

    /// Rend visible l'onglet actif. A appeler apres un detachement.
    pub fn refresh_visibility(&self) {
        for tab in &self.entries {
            if let Some(view) = &tab.view {
                View::from(view).set_visible(i32::from(Some(tab.id) == self.active));
            }
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

    /// Tout ce qu'il faut pour retrouver les onglets apres une relance.
    pub fn to_snapshot(&self) -> crate::restart::Snapshot {
        let active = self
            .active
            .and_then(|id| self.entries.iter().position(|tab| tab.id == id))
            .unwrap_or(0);
        crate::restart::Snapshot {
            tabs: self
                .entries
                .iter()
                .filter(|tab| !tab.history.is_empty())
                .map(|tab| crate::restart::TabSnapshot {
                    history: tab.history.clone(),
                    position: tab.position,
                })
                .collect(),
            active,
        }
    }

    /// Deplace un onglet a une nouvelle position dans la liste.
    pub fn move_to(&mut self, id: TabId, to: usize) {
        let Some(from) = self.entries.iter().position(|tab| tab.id == id) else { return };
        let tab = self.entries.remove(from);
        self.entries.insert(to.min(self.entries.len()), tab);
    }

    /// Rend a un onglet le fil qu'il avait avant la relance.
    pub fn restore_history(&mut self, id: TabId, history: Vec<String>, position: usize) {
        let Some(tab) = self.get_mut(id) else { return };
        tab.position = position.min(history.len().saturating_sub(1));
        tab.history = history;
    }

    /// L'etat de tous les onglets, dans l'ordre d'affichage.
    pub fn snapshot(&self) -> Vec<TabView> {
        self.entries.iter().map(Tab::to_view).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
