//! Extensions — installer, activer et recenser les extensions du catalogue Chrome.
//!
//! Chromium ne sait charger des extensions qu'au demarrage : installer ou retirer une
//! extension demande donc de relancer le navigateur. Ce domaine gere le disque et
//! l'inventaire ; le chargement lui-meme appartient a la coque.

pub mod catalog;
pub mod crx;
pub mod store;

use std::path::{Path, PathBuf};
use store::Extension;
use tracing::info;

/// Le dossier des extensions et ce qu'on peut y faire.
pub struct Extensions {
    root: PathBuf,
}

impl Extensions {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Installe une extension depuis le catalogue. Accepte un identifiant ou une adresse.
    /// Prend effet au prochain demarrage du navigateur.
    pub fn install(&self, input: &str) -> anyhow::Result<Extension> {
        let id = catalog::extract_id(input)
            .ok_or_else(|| anyhow::anyhow!("aucun identifiant d'extension dans « {input} »"))?;
        let package = catalog::download(&id)?;
        let dir = store::extension_dir(&self.root, &id);
        let files = crx::unpack(&package, &dir)?;

        let (raw_name, version) = store::read_manifest(&dir)
            .ok_or_else(|| anyhow::anyhow!("le paquet de {id} n'a pas de manifeste lisible"))?;
        let name = store::resolve_name(&dir, &raw_name);
        info!(%id, %name, %version, fichiers = files, "extension installee");
        Ok(Extension { id, name, version, enabled: true })
    }

    pub fn list(&self) -> Vec<Extension> {
        store::list(&self.root)
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) -> anyhow::Result<()> {
        store::set_enabled(&self.root, id, enabled)
    }

    pub fn remove(&self, id: &str) -> anyhow::Result<()> {
        store::remove(&self.root, id)
    }

    /// Les dossiers a charger au demarrage, dans l'ordre.
    pub fn loadable(&self) -> Vec<String> {
        self.list()
            .into_iter()
            .filter(|extension| extension.enabled)
            .filter_map(|extension| {
                store::extension_dir(&self.root, &extension.id)
                    .to_str()
                    .map(str::to_owned)
            })
            .collect()
    }
}
