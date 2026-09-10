//! Bouclier — blocage des publicites et des traqueurs, integre au navigateur.
//!
//! Le domaine expose un seul point d'entree, [`Shield`], que le reste du navigateur
//! interroge : une decision par requete reseau, un traitement par page.

pub mod catalog;
pub mod engine;
pub mod exceptions;
pub mod resources;
pub mod tally;
pub mod verdict;

use catalog::Subscription;
use engine::{EngineError, FilterEngine};
use exceptions::{allowlist_path, Allowlist};
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use tally::{Tally, TallySnapshot};
use tracing::{info, warn};
use verdict::{PageTreatment, Verdict};

/// Etat global du bouclier, partage par tous les onglets.
pub struct Shield {
    engine: RwLock<Option<FilterEngine>>,
    allowlist: RwLock<Allowlist>,
    tally: Tally,
    data_dir: PathBuf,
    lists_dir: PathBuf,
    subscriptions: RwLock<Vec<Subscription>>,
    enabled: RwLock<bool>,
}

impl Shield {
    /// Prepare le bouclier sans encore charger les listes.
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();
        let lists_dir = data_dir.join("filter-lists");
        let allowlist = Allowlist::load(&allowlist_path(&data_dir));
        let disabled = Self::load_subscription_choices(&data_dir);
        let mut subscriptions = catalog::default_subscriptions();
        for sub in subscriptions.iter_mut() {
            if disabled.iter().any(|id| *id == sub.id) {
                sub.enabled = false;
            }
        }
        Self {
            engine: RwLock::new(None),
            allowlist: RwLock::new(allowlist),
            tally: Tally::default(),
            data_dir,
            lists_dir,
            subscriptions: RwLock::new(subscriptions),
            enabled: RwLock::new(true),
        }
    }

    /// Charge le moteur : cache binaire si disponible, sinon analyse des listes.
    pub fn load(&self) -> Result<(), EngineError> {
        let pack = resources::load_pack(&resources::pack_path(&self.data_dir));
        let cache = engine::cache_path(&self.data_dir);

        if cache.exists() {
            match FilterEngine::from_cache(&cache, pack.clone()) {
                Ok(engine) => {
                    *self.engine.write() = Some(engine);
                    return Ok(());
                }
                Err(err) => warn!(%err, "cache inutilisable, reconstruction depuis les listes"),
            }
        }

        let subs = self.subscriptions.read().clone();
        let engine = FilterEngine::build(&self.lists_dir, &subs, pack)?;
        if let Err(err) = engine.write_cache(&cache) {
            warn!(%err, "cache non ecrit — le prochain demarrage sera plus lent");
        }
        *self.engine.write() = Some(engine);
        Ok(())
    }

    /// Telecharge les listes manquantes ou perimees, puis reconstruit le moteur si besoin.
    pub fn refresh_lists(&self, force: bool) -> anyhow::Result<usize> {
        let subs = self.subscriptions.read().clone();
        let mut refreshed = 0usize;
        for sub in subs.iter().filter(|s| s.enabled) {
            let path = catalog::list_path(&self.lists_dir, &sub.id);
            if !force && !catalog::needs_refresh(&path) {
                continue;
            }
            match catalog::fetch_list(sub, &self.lists_dir) {
                Ok(_) => refreshed += 1,
                Err(err) => warn!(id = %sub.id, %err, "liste non rafraichie, ancienne version conservee"),
            }
        }
        if refreshed > 0 {
            let _ = std::fs::remove_file(engine::cache_path(&self.data_dir));
            let pack = resources::load_pack(&resources::pack_path(&self.data_dir));
            let engine = FilterEngine::build(&self.lists_dir, &subs, pack)?;
            if let Err(err) = engine.write_cache(&engine_cache(&self.data_dir)) {
                warn!(%err, "cache non reecrit");
            }
            *self.engine.write() = Some(engine);
            info!(listes = refreshed, "listes rafraichies, moteur reconstruit");
        }
        Ok(refreshed)
    }

    /// Decision sur une requete. Toute reponse autre que `Allow` incremente les compteurs.
    pub fn decide(&self, tab_id: u32, url: &str, source_url: &str, resource_type: &str, method: &str) -> Verdict {
        if !self.is_active_for(source_url) {
            return Verdict::Allow;
        }
        let guard = self.engine.read();
        let Some(engine) = guard.as_ref() else {
            return Verdict::Allow;
        };
        let verdict = engine.decide(url, source_url, resource_type, method);
        if verdict != Verdict::Allow {
            self.tally.record_block(tab_id);
        }
        verdict
    }

    /// Masquage et scriptlets pour une page sur le point de s'afficher.
    pub fn treat_page(&self, url: &str) -> PageTreatment {
        if !self.is_active_for(url) {
            return PageTreatment::default();
        }
        self.engine.read().as_ref().map(|e| e.treat_page(url)).unwrap_or_default()
    }

    /// Bascule le bouclier pour le site courant. Renvoie l'etat resultant.
    pub fn toggle_site(&self, url: &str) -> bool {
        let Some(host) = hostname_of(url) else {
            return true;
        };
        let active = self.allowlist.write().toggle(&host);
        if let Err(err) = self.allowlist.read().save(&allowlist_path(&self.data_dir)) {
            warn!(%err, "liste blanche non enregistree");
        }
        active
    }

    /// Bascule globale du bouclier.
    pub fn set_enabled(&self, on: bool) {
        *self.enabled.write() = on;
        info!(actif = on, "bouclier bascule globalement");
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }

    /// Vrai si le bouclier doit s'appliquer a cette page.
    pub fn is_active_for(&self, url: &str) -> bool {
        if !*self.enabled.read() {
            return false;
        }
        match hostname_of(url) {
            Some(host) => !self.allowlist.read().covers(&host),
            None => true,
        }
    }

    pub fn tally(&self, tab_id: u32) -> TallySnapshot {
        self.tally.snapshot(tab_id)
    }

    pub fn reset_tab(&self, tab_id: u32) {
        self.tally.reset_tab(tab_id);
    }

    pub fn subscriptions(&self) -> Vec<Subscription> {
        self.subscriptions.read().clone()
    }

    /// Nombre de regles chargees par liste, pour celles qui sont actives.
    pub fn rules_per_list(&self) -> Vec<(String, Option<usize>)> {
        self.subscriptions
            .read()
            .iter()
            .map(|sub| {
                let count = sub.enabled.then(|| {
                    std::fs::read_to_string(catalog::list_path(&self.lists_dir, &sub.id))
                        .map(|text| text.lines().count())
                        .unwrap_or(0)
                });
                (sub.id.clone(), count)
            })
            .collect()
    }

    /// Active ou desactive une liste, puis reconstruit le moteur.
    /// Renvoie faux si la liste est inconnue.
    pub fn set_list_enabled(&self, id: &str, enabled: bool) -> bool {
        {
            let mut subs = self.subscriptions.write();
            let Some(sub) = subs.iter_mut().find(|sub| sub.id == id) else {
                warn!(%id, "liste inconnue");
                return false;
            };
            if sub.enabled == enabled {
                return true;
            }
            sub.enabled = enabled;
        }
        self.save_subscriptions();
        if let Err(err) = self.rebuild() {
            warn!(%err, "moteur non reconstruit apres changement de liste");
        }
        true
    }

    /// Reconstruit le moteur depuis les listes actives, en jetant le cache.
    pub fn rebuild(&self) -> Result<(), EngineError> {
        let _ = std::fs::remove_file(engine::cache_path(&self.data_dir));
        let subs = self.subscriptions.read().clone();
        let pack = resources::load_pack(&resources::pack_path(&self.data_dir));
        let engine = FilterEngine::build(&self.lists_dir, &subs, pack)?;
        if let Err(err) = engine.write_cache(&engine::cache_path(&self.data_dir)) {
            warn!(%err, "cache non reecrit");
        }
        *self.engine.write() = Some(engine);
        info!("moteur reconstruit");
        Ok(())
    }

    /// Date du dernier rafraichissement, lue sur la liste la plus recemment ecrite.
    pub fn refreshed_at(&self) -> Option<i64> {
        self.subscriptions
            .read()
            .iter()
            .filter_map(|sub| std::fs::metadata(catalog::list_path(&self.lists_dir, &sub.id)).ok())
            .filter_map(|meta| meta.modified().ok())
            .filter_map(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|since| since.as_secs() as i64)
            .max()
    }

    fn save_subscriptions(&self) {
        let path = self.data_dir.join("shield-lists.json");
        let subs = self.subscriptions.read();
        let disabled: Vec<&str> =
            subs.iter().filter(|sub| !sub.enabled).map(|sub| sub.id.as_str()).collect();
        match serde_json::to_vec_pretty(&disabled) {
            Ok(bytes) => {
                if let Err(err) = std::fs::write(&path, bytes) {
                    warn!(%err, "choix de listes non enregistre");
                }
            }
            Err(err) => warn!(%err, "choix de listes non serialisable"),
        }
    }

    /// Reprend les listes que l'utilisateur avait desactivees.
    fn load_subscription_choices(data_dir: &Path) -> Vec<String> {
        let path = data_dir.join("shield-lists.json");
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }
}

fn engine_cache(data_dir: &Path) -> PathBuf {
    engine::cache_path(data_dir)
}

/// Extrait le nom d'hote d'une URL sans dependre d'un analyseur complet.
fn hostname_of(url: &str) -> Option<String> {
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let host = rest.split(['/', '?', '#']).next()?;
    let host = host.rsplit_once('@').map(|(_, h)| h).unwrap_or(host);
    let host = host.split_once(':').map(|(h, _)| h).unwrap_or(host);
    if host.is_empty() { None } else { Some(host.to_ascii_lowercase()) }
}
