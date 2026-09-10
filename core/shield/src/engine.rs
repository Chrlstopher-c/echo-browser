//! Responsabilite : construire et interroger le moteur de filtrage, et le recharger vite au demarrage.

use crate::catalog::{list_path, Subscription};
use crate::verdict::{PageTreatment, Verdict};
use adblock::lists::{FilterSet, ParseOptions};
use adblock::request::Request;
use adblock::resources::Resource;
use adblock::Engine as AdEngine;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{info, warn};

/// Version du format de cache. A incrementer des que la composition du moteur change.
const CACHE_FORMAT: u8 = 1;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("aucune liste active : le bouclier n'a rien a appliquer")]
    NoActiveList,
    #[error("cache illisible : {0}")]
    Cache(String),
    #[error("acces disque : {0}")]
    Io(#[from] std::io::Error),
}

/// Le moteur de filtrage charge en memoire.
pub struct FilterEngine {
    inner: AdEngine,
    rules_loaded: usize,
}

impl FilterEngine {
    /// Construit le moteur en analysant les listes presentes sur disque.
    pub fn build(dir: &Path, subs: &[Subscription], resources: Vec<Resource>) -> Result<Self, EngineError> {
        let started = Instant::now();
        let mut set = FilterSet::new(false);
        let mut rules_loaded = 0usize;

        for sub in subs.iter().filter(|s| s.enabled) {
            let path = list_path(dir, &sub.id);
            match std::fs::read_to_string(&path) {
                Ok(text) => {
                    rules_loaded += text.lines().count();
                    set.add_filter_list(text, ParseOptions::default());
                }
                Err(err) => warn!(id = %sub.id, %err, "liste ignoree, fichier absent ou illisible"),
            }
        }
        if rules_loaded == 0 {
            return Err(EngineError::NoActiveList);
        }

        let mut inner = AdEngine::new_with_filter_set(set);
        let resource_count = resources.len();
        inner.use_resources(resources);
        info!(regles = rules_loaded, ressources = resource_count, duree = ?started.elapsed(), "moteur construit");
        Ok(Self { inner, rules_loaded })
    }

    /// Recharge le moteur depuis le cache binaire — bien plus rapide que de reanalyser les listes.
    pub fn from_cache(path: &Path, resources: Vec<Resource>) -> Result<Self, EngineError> {
        let started = Instant::now();
        let blob = std::fs::read(path)?;
        let (&version, payload) = blob.split_first().ok_or_else(|| EngineError::Cache("cache vide".into()))?;
        if version != CACHE_FORMAT {
            return Err(EngineError::Cache(format!("format {version} au lieu de {CACHE_FORMAT}")));
        }

        let mut inner = AdEngine::default();
        inner.deserialize(payload).map_err(|err| EngineError::Cache(format!("{err:?}")))?;
        inner.use_resources(resources);
        info!(duree = ?started.elapsed(), "moteur recharge depuis le cache");
        Ok(Self { inner, rules_loaded: 0 })
    }

    /// Ecrit le cache binaire pour le prochain demarrage.
    pub fn write_cache(&self, path: &Path) -> Result<usize, EngineError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut blob = Vec::with_capacity(16 * 1024 * 1024);
        blob.push(CACHE_FORMAT);
        blob.extend_from_slice(&self.inner.serialize());
        std::fs::write(path, &blob)?;
        info!(octets = blob.len(), "cache du bouclier ecrit");
        Ok(blob.len())
    }

    /// Decision sur une requete reseau.
    pub fn decide(&self, url: &str, source_url: &str, resource_type: &str, method: &str) -> Verdict {
        let request = match Request::new(url, source_url, resource_type, method) {
            Ok(request) => request,
            Err(err) => {
                warn!(%url, ?err, "requete non analysable, laissee passer");
                return Verdict::Allow;
            }
        };
        let result = self.inner.check_network_request(&request);
        if let Some(resource) = result.redirect {
            return Verdict::Redirect { resource };
        }
        if result.should_block() {
            return Verdict::Block { rule: result.filter.map(|f| format!("{f:?}")) };
        }
        Verdict::Allow
    }

    /// Masquage et scriptlets a appliquer a une page.
    pub fn treat_page(&self, url: &str) -> PageTreatment {
        let res = self.inner.url_cosmetic_resources(url);
        PageTreatment {
            hide_selectors: res.hide_selectors.into_iter().collect(),
            procedural_actions: res.procedural_actions.into_iter().collect(),
            injected_script: res.injected_script,
        }
    }

    /// Selecteurs generiques a masquer pour les classes et identifiants reellement presents dans la page.
    pub fn treat_dom(&self, classes: &[String], ids: &[String], exceptions: &[String]) -> Vec<String> {
        let exceptions = exceptions.iter().cloned().collect();
        self.inner
            .hidden_class_id_selectors(classes, ids, &exceptions)
            .into_iter()
            .collect()
    }

    pub fn rules_loaded(&self) -> usize {
        self.rules_loaded
    }
}

/// Emplacement du cache binaire du moteur.
pub fn cache_path(data_dir: &Path) -> PathBuf {
    data_dir.join("shield-engine.bin")
}
