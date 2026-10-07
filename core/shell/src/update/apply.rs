//! La bascule vers la version preparee et le retour en arriere vivent dans le lanceur `echo-browser.sh` de l'archive :
//! un binaire casse ne saurait pas revenir lui-meme a l'ancienne version. Ici, seulement ce qu'Echo doit dire au lanceur.

/// Fichier pose par le lanceur apres une bascule ; tant qu'il existe, la nouvelle version est a l'essai.
const TRIAL_FILE: &str = ".essai-demarrage";

/// Le moteur a demarre : la version installee est validee.
pub fn confirm_started() {
    if let Some(install) = super::install_dir() {
        let _ = std::fs::remove_file(install.join(TRIAL_FILE));
    }
}

/// Le programme a relancer : le lanceur de l'archive installee (il applique une mise a jour preparee), sinon `None`.
pub fn launcher() -> Option<std::path::PathBuf> {
    super::install_dir().map(|dir| dir.join("echo-browser.sh")).filter(|p| p.is_file())
}
