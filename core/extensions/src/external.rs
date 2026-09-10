//! Responsabilite : declarer une extension a Chromium pour qu'il l'installe lui-meme.
//!
//! Trois voies ont ete essayees le 10/09/2026, mesure a l'appui :
//!
//! - **La fiche du catalogue** ouverte dans un onglet : la page reconnait qu'elle ne
//!   parle pas au vrai Chrome et renvoie vers l'application installee sur la machine.
//!   L'extension atterrit dans un autre navigateur.
//! - **Le paquet depaquete par nos soins**, charge par `--load-extension` : l'extension
//!   est listee, annoncee active, et pourtant toutes ses adresses repondent
//!   `ERR_BLOCKED_BY_CLIENT`. Ni le mode developpeur ni `--disable-extensions-except`
//!   n'y changent rien.
//! - **La declaration externe**, celle-ci : un fichier par extension dans
//!   « External Extensions », et Chromium telecharge, verifie et installe dans son
//!   profil au demarrage suivant. C'est la seule qui marche, et elle donne en prime
//!   les mises a jour automatiques.

use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Service de mise a jour du catalogue Chrome. C'est lui qui sert les paquets.
const UPDATE_URL: &str = "https://clients2.google.com/service/update2/crx";

/// Dossier ou Chromium lit les extensions a installer, sous la racine du profil.
pub fn declarations_dir(profile_root: &Path) -> PathBuf {
    profile_root.join("External Extensions")
}

/// Chemin de la declaration d'une extension.
fn declaration(profile_root: &Path, id: &str) -> PathBuf {
    declarations_dir(profile_root).join(format!("{id}.json"))
}

/// Declare une extension. Chromium l'installe au demarrage suivant.
pub fn declare(profile_root: &Path, id: &str) -> anyhow::Result<()> {
    let dir = declarations_dir(profile_root);
    std::fs::create_dir_all(&dir)?;
    let body = serde_json::json!({ "external_update_url": UPDATE_URL });
    std::fs::write(declaration(profile_root, id), serde_json::to_vec_pretty(&body)?)?;
    info!(%id, "extension declaree, installation au prochain demarrage");
    Ok(())
}

/// Retire la declaration. Chromium desinstalle ce qui n'est plus declare.
pub fn withdraw(profile_root: &Path, id: &str) -> anyhow::Result<()> {
    let path = declaration(profile_root, id);
    if path.exists() {
        std::fs::remove_file(&path)?;
        info!(%id, "declaration retiree");
    }
    Ok(())
}

/// Les extensions declarees, qu'elles soient deja installees ou non.
pub fn declared(profile_root: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(declarations_dir(profile_root)) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()?.to_str()? != "json" {
                return None;
            }
            Some(path.file_stem()?.to_str()?.to_string())
        })
        .collect()
}

/// Vrai si l'identifiant a la forme d'un identifiant du catalogue : 32 lettres de a a p.
pub fn is_store_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|b| (b'a'..=b'p').contains(&b))
}

/// Refuse une declaration dont l'identifiant n'a pas la bonne forme : le nom du fichier
/// vient de la saisie de l'utilisateur, il ne doit jamais designer un autre chemin.
pub fn check_id(id: &str) -> anyhow::Result<()> {
    if !is_store_id(id) {
        warn!(%id, "identifiant d'extension refuse");
        anyhow::bail!("« {id} » n'est pas un identifiant du catalogue (32 lettres de a a p)");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declare_puis_retire() {
        let racine = std::env::temp_dir().join(format!("echo-ext-{}", std::process::id()));
        let id = "ghmbeldphafepmbegfdlkpapadhbakde";
        declare(&racine, id).expect("declaration");
        assert_eq!(declared(&racine), vec![id.to_string()]);
        let contenu = std::fs::read_to_string(racine.join("External Extensions").join(format!("{id}.json")))
            .expect("relecture");
        assert!(contenu.contains("external_update_url"), "{contenu}");
        withdraw(&racine, id).expect("retrait");
        assert!(declared(&racine).is_empty());
        std::fs::remove_dir_all(&racine).ok();
    }

    #[test]
    fn un_identifiant_malforme_est_refuse() {
        assert!(check_id("../../evasion").is_err());
        assert!(check_id("ghmbeldphafepmbegfdlkpapadhbakdz").is_err(), "z sort de l'alphabet");
        assert!(check_id("ghmbeldphafepmbegfdlkpapadhbakde").is_ok());
    }
}
