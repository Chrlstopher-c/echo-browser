//! Extensions — installer, activer et recenser les extensions du catalogue Chrome.
//!
//! Chromium ne sait charger des extensions qu'au demarrage : installer ou retirer une
//! extension demande donc de relancer le navigateur. Ce domaine gere le disque et
//! l'inventaire ; le chargement lui-meme appartient a la coque.

pub mod action;
pub mod catalog;
pub mod crx;
pub mod profile;
pub mod store;

use std::path::{Path, PathBuf};
use store::Extension;
use tracing::info;

/// Les extensions du navigateur : celles que Chromium gere dans son profil, et le
/// dossier des extensions que nous chargeons nous-memes au demarrage.
pub struct Extensions {
    /// Dossier des extensions depaquetees par nos soins.
    root: PathBuf,
    /// Profil Chromium, source de verite de ce qui tourne reellement.
    profile: PathBuf,
}

impl Extensions {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let profile = root.parent().map(|dir| dir.join("profile").join("Default")).unwrap_or_default();
        Self { root, profile }
    }

    /// Precise ou vit le profil Chromium quand il n'est pas a cote du dossier d'extensions.
    pub fn with_profile(mut self, profile: impl Into<PathBuf>) -> Self {
        self.profile = profile.into();
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Adresse a ouvrir pour installer une extension par le parcours de Chromium.
    pub fn store_page(id: &str) -> String {
        profile::store_page(id)
    }

    /// Installe une extension depuis le catalogue. Accepte un identifiant ou une adresse.
    /// Prend effet au prochain demarrage du navigateur.
    pub fn install(&self, input: &str) -> anyhow::Result<Extension> {
        let id = catalog::extract_id(input)
            .ok_or_else(|| anyhow::anyhow!("aucun identifiant d'extension dans « {input} »"))?;
        let package = catalog::download(&id)?;
        let dir = store::extension_dir(&self.root, &id);
        let files = crx::unpack(&package, &dir)?;

        let manifest = store::read_manifest_json(&dir)
            .ok_or_else(|| anyhow::anyhow!("le paquet de {id} n'a pas de manifeste lisible"))?;
        let raw_name = manifest
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("le manifeste de {id} n'a pas de nom"))?;
        let version =
            manifest.get("version").and_then(|v| v.as_str()).unwrap_or("0").to_string();
        let name = store::resolve_name(&dir, raw_name);
        info!(%id, %name, %version, fichiers = files, "extension installee");
        Ok(Extension {
            id,
            name,
            version,
            enabled: true,
            removable: true,
            from_command_line: true,
            action: action::Action::from_manifest(&manifest),
            dir,
        })
    }

    /// L'inventaire complet : ce que Chromium connait, complete par nos propres paquets.
    ///
    /// Les deux sources sont necessaires. Le profil sait ce qui tourne vraiment, mais
    /// n'y decrit pas toujours les extensions passees en ligne de commande ; notre
    /// dossier les decrit, mais ignore celles installees depuis le catalogue.
    pub fn list(&self) -> Vec<Extension> {
        let mut found = profile::list(&self.profile);
        for ours in store::list(&self.root) {
            if !found.iter().any(|known| known.id == ours.id) {
                found.push(ours);
            }
        }
        found.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        found
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) -> anyhow::Result<()> {
        store::set_enabled(&self.root, id, enabled)
    }

    pub fn remove(&self, id: &str) -> anyhow::Result<()> {
        store::remove(&self.root, id)
    }

    /// Les dossiers a passer a Chromium au demarrage.
    ///
    /// Uniquement nos propres paquets : celles du catalogue sont deja installees dans
    /// le profil, et les redonner en ligne de commande creerait un doublon.
    pub fn loadable(&self) -> Vec<String> {
        store::list(&self.root)
            .into_iter()
            .filter(|extension| extension.enabled)
            .filter_map(|extension| {
                let dir = store::extension_dir(&self.root, &extension.id);
                dir.is_dir().then(|| dir.to_str().map(str::to_owned))?
            })
            .collect()
    }
}
