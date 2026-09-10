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
        // Notre propre registre a le dernier mot sur l'activation, y compris pour les
        // extensions du catalogue : Chromium refuse qu'on ecrive dans ses preferences
        // (mesure du 2026-09-10, la valeur est remise a l'identique au demarrage), mais
        // il accepte de ne charger que ce qu'on lui designe en ligne de commande.
        let disabled = store::read_disabled(&self.root);
        for extension in &mut found {
            extension.enabled = extension.enabled && !disabled.contains(&extension.id);
        }
        found.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        found
    }

    /// Les dossiers des extensions a laisser vivre, quand au moins une est ecartee.
    ///
    /// Chromium ne sait pas desactiver une extension a chaud : il sait n'en charger
    /// qu'une liste au demarrage. Une bascule attend donc la relance — c'est deja ce
    /// que l'interface annonce pour nos propres paquets.
    pub fn enabled_paths(&self) -> Option<Vec<String>> {
        let all = self.list();
        if all.iter().all(|extension| extension.enabled) {
            return None;
        }
        Some(
            all.into_iter()
                .filter(|extension| extension.enabled)
                .filter(|extension| extension.dir.is_dir())
                .filter_map(|extension| extension.dir.to_str().map(str::to_owned))
                .collect(),
        )
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) -> anyhow::Result<()> {
        store::set_enabled(&self.root, id, enabled)
    }

    /// Retire une extension, d'ou qu'elle vienne.
    ///
    /// Nos paquets vivent dans notre dossier ; celles du catalogue vivent dans le
    /// profil, ou Chromium les a depaquetees. Dans les deux cas on efface le dossier :
    /// Chromium constate l'absence au demarrage suivant et oublie l'entree. Il n'existe
    /// pas d'autre voie — ses preferences refusent nos ecritures.
    pub fn remove(&self, id: &str) -> anyhow::Result<()> {
        let known = self.list().into_iter().find(|extension| extension.id == id);
        if let Some(extension) = known {
            let dir = &extension.dir;
            // La suppression ne sort jamais de nos deux racines connues.
            let sien = dir.starts_with(&self.root) || dir.starts_with(&self.profile);
            if sien && dir.is_dir() {
                std::fs::remove_dir_all(dir)?;
                info!(%id, ?dir, "extension retiree");
            } else if !sien {
                anyhow::bail!("dossier inattendu pour {id} : {}", dir.display());
            }
        }
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
