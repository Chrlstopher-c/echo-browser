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
    /// Defilement vertical de la page, en pixels : rendu a l'onglet quand il se reveille.
    pub scroll: i32,
    /// Defilement a rejouer des que la page rechargee a fini de charger.
    pub pending_scroll: Option<i32>,
    /// Adresse de l'icone du site, quand la page en a declare une.
    pub favicon: Option<String>,
    /// Dernier moment ou l'onglet a ete l'onglet actif.
    pub last_active: Instant,
    /// La memoire JavaScript de la page a deja ete purgee depuis qu'elle est passee en arriere-plan.
    pub trimmed: bool,
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
    /// Dossier d'onglets ou l'onglet est range (identifiant).
    pub folder: Option<String>,
    /// Conteneur de l'onglet (cookies et comptes a part), `None` pour le contexte commun.
    pub container: Option<String>,
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
            favicon: self.favicon.clone(),
            security: security_of(&self.url),
            pinned: self.pinned,
            folder: self.folder.clone(),
            container: self.container.clone(),
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
            scroll: 0,
            pending_scroll: None,
            favicon: None,
            last_active: Instant::now(),
            trimmed: false,
            title: url.to_string(),
            url: url.to_string(),
            loading: true,
            history: vec![url.to_string()],
            position: 0,
            pinned: false,
            folder: None,
            container: None,
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
            tab.trimmed = false;
        }
        self.refresh_visibility();
    }

    pub fn exists(&self, id: TabId) -> bool {
        self.entries.iter().any(|tab| tab.id == id)
    }

    pub fn find_by_url(&self, url: &str) -> Option<TabId> {
        self.entries.iter().find(|tab| tab.url == url).map(|tab| tab.id)
    }

    /// Les pages chargees de tous les onglets vivants.
    pub fn main_frames(&self) -> Vec<Frame> {
        self.entries.iter().filter_map(|tab| tab.browser()?.main_frame()).collect()
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

    /// Les pages d'arriere-plan inactives depuis `idle` dont la memoire n'a pas encore ete purgee.
    pub fn take_trim_targets(&mut self, idle: Duration) -> Vec<cef::Browser> {
        let active = self.active;
        let mut targets = Vec::new();
        for tab in self.entries.iter_mut() {
            if Some(tab.id) == active || tab.asleep || tab.trimmed || tab.last_active.elapsed() < idle {
                continue;
            }
            if let Some(browser) = tab.browser() {
                tab.trimmed = true;
                targets.push(browser);
            }
        }
        targets
    }

    /// Nombre d'onglets dont la page est chargee en memoire.
    pub fn live_count(&self) -> usize {
        self.entries.iter().filter(|tab| !tab.asleep).count()
    }

    /// Les onglets inactifs depuis au moins `idle`, bons a endormir.
    pub fn sleep_candidates(&self, idle: Duration, never: &[String]) -> Vec<TabId> {
        self.entries
            .iter()
            .filter(|tab| {
                !tab.asleep
                    && Some(tab.id) != self.active
                    && !host_listed(&tab.url, never)
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

    pub fn container_of(&self, id: TabId) -> Option<String> {
        self.entries.iter().find(|tab| tab.id == id).and_then(|tab| tab.container.clone())
    }

    /// Rend une vue neuve a un onglet endormi.
    pub fn wake_with(&mut self, id: TabId, view: BrowserView) {
        let Some(tab) = self.get_mut(id) else { return };
        tab.view = Some(view);
        tab.asleep = false;
        tab.loading = true;
        tab.pending_scroll = (tab.scroll > 0).then_some(tab.scroll);
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

    /// Ajoute un onglet endormi, restitue sans etre charge : il se recharge quand on le selectionne.
    pub fn adopt_asleep(&mut self, snapshot: &crate::restart::TabSnapshot) {
        let Some(url) = snapshot.current() else { return };
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(Tab {
            id,
            view: None,
            asleep: true,
            dirty: false,
            scroll: snapshot.scroll,
            pending_scroll: None,
            favicon: snapshot.favicon.clone(),
            last_active: Instant::now(),
            trimmed: false,
            title: if snapshot.title.is_empty() { url.to_string() } else { snapshot.title.clone() },
            url: url.to_string(),
            loading: false,
            history: snapshot.history.clone(),
            position: snapshot.position.min(snapshot.history.len().saturating_sub(1)),
            pinned: snapshot.pinned,
            folder: snapshot.folder.clone(),
            container: snapshot.container.clone(),
            zoom: 1.0,
            audible: false,
        });
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
                    pinned: tab.pinned,
                    folder: tab.folder.clone(),
                    container: tab.container.clone(),
                    scroll: tab.scroll,
                    title: tab.title.clone(),
                    favicon: tab.favicon.clone(),
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

/// Vrai si l'adresse appartient a l'un des sites de la liste (le site lui-meme ou un de ses sous-domaines).
fn host_listed(url: &str, hosts: &[String]) -> bool {
    let host = url.split("://").nth(1).and_then(|rest| rest.split(['/', '?', '#', ':']).next()).unwrap_or("");
    hosts.iter().any(|entry| host == entry || host.ends_with(&format!(".{entry}")))
}
