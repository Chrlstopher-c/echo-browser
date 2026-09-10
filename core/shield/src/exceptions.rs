//! Responsabilite : retenir les sites ou l'utilisateur a desactive le bouclier.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Domaines exemptes de filtrage, a la demande de l'utilisateur.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Allowlist {
    domains: BTreeSet<String>,
}

impl Allowlist {
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|err| {
                warn!(?path, %err, "liste blanche illisible, repartie a vide");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }

    /// Vrai si le domaine, ou l'un de ses parents, est exempte.
    pub fn covers(&self, hostname: &str) -> bool {
        let mut candidate = hostname;
        loop {
            if self.domains.contains(candidate) {
                return true;
            }
            match candidate.split_once('.') {
                Some((_, rest)) if rest.contains('.') => candidate = rest,
                _ => return false,
            }
        }
    }

    /// Inverse l'etat d'un domaine. Renvoie l'etat du bouclier apres bascule.
    pub fn toggle(&mut self, hostname: &str) -> bool {
        if self.domains.remove(hostname) {
            info!(%hostname, "bouclier reactive");
            true
        } else {
            self.domains.insert(hostname.to_string());
            info!(%hostname, "bouclier desactive pour ce site");
            false
        }
    }

    pub fn domains(&self) -> impl Iterator<Item = &String> {
        self.domains.iter()
    }
}

/// Liste blanche partagee entre les threads du navigateur.
pub type SharedAllowlist = RwLock<Allowlist>;

pub fn allowlist_path(data_dir: &Path) -> PathBuf {
    data_dir.join("shield-allowlist.json")
}
