//! Responsabilite : quelles extensions appartiennent a quel profil d'Echo. Un profil est une identite : ses extensions
//! ne se partagent pas. Chromium, lui, installe une extension declaree dans tous ses profils ; ce registre dit lesquelles
//! garder actives dans chacun (le pont interne applique la regle dans chaque profil Chromium).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub type Registry = BTreeMap<String, Vec<String>>;

const FILE: &str = "profils.json";

pub fn path(root: &Path) -> PathBuf {
    root.join(FILE)
}

/// Le registre. Absent (premier lancement apres la separation) : tout ce qui est installe va au profil `default`.
pub fn load(root: &Path, default: &str, installed: impl FnOnce() -> Vec<String>) -> Registry {
    match std::fs::read(path(root)).ok().and_then(|raw| serde_json::from_slice(&raw).ok()) {
        Some(registry) => registry,
        None => BTreeMap::from([(default.to_string(), installed())]),
    }
}

pub fn save(root: &Path, registry: &Registry) -> anyhow::Result<()> {
    std::fs::create_dir_all(root)?;
    std::fs::write(path(root), serde_json::to_vec_pretty(registry)?)?;
    Ok(())
}

pub fn ids_for<'a>(registry: &'a Registry, profile: &str) -> &'a [String] {
    registry.get(profile).map_or(&[], Vec::as_slice)
}

/// Ajoute l'extension au profil. Faux si elle y etait deja.
pub fn add(registry: &mut Registry, profile: &str, id: &str) -> bool {
    let ids = registry.entry(profile.to_string()).or_default();
    if ids.iter().any(|known| known == id) {
        return false;
    }
    ids.push(id.to_string());
    true
}

/// Retire l'extension du profil. Vrai si un autre profil la garde encore.
pub fn remove(registry: &mut Registry, profile: &str, id: &str) -> bool {
    if let Some(ids) = registry.get_mut(profile) {
        ids.retain(|known| known != id);
    }
    registry.values().any(|ids| ids.iter().any(|known| known == id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premier_lancement_tout_va_au_profil_principal() {
        let root = std::env::temp_dir().join(format!("echo-profils-{}", std::process::id()));
        let registry = load(&root, "graphite", || vec!["a".into(), "b".into()]);
        assert_eq!(ids_for(&registry, "graphite"), ["a", "b"]);
        assert!(ids_for(&registry, "autre").is_empty());
    }

    #[test]
    fn une_extension_retiree_d_un_profil_reste_dans_l_autre() {
        let mut registry = Registry::new();
        assert!(add(&mut registry, "p1", "x"));
        assert!(!add(&mut registry, "p1", "x"));
        add(&mut registry, "p2", "x");
        assert!(remove(&mut registry, "p1", "x"), "p2 la garde");
        assert!(!remove(&mut registry, "p2", "x"), "plus personne");
        assert!(ids_for(&registry, "p1").is_empty());
    }

    #[test]
    fn le_registre_survit_a_l_ecriture() {
        let root = std::env::temp_dir().join(format!("echo-profils-ecrit-{}", std::process::id()));
        let mut registry = Registry::new();
        add(&mut registry, "p1", "x");
        save(&root, &registry).unwrap();
        assert_eq!(load(&root, "graphite", Vec::new), registry);
        let _ = std::fs::remove_dir_all(root);
    }
}
