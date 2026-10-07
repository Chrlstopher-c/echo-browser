//! Responsabilite : relancer le navigateur sans perdre ce qui etait ouvert.
//!
//! Chromium ne se reinitialise pas dans un processus vivant : appliquer un changement
//! d'extensions demande de repartir. Pour l'utilisateur, cela doit rester un clignotement,
//! pas une fermeture — d'ou la sauvegarde des onglets avant, et leur reprise apres.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn};

/// Positionne quand une relance a ete demandee : le programme repart apres l'arret de Chromium.
static REQUESTED: AtomicBool = AtomicBool::new(false);

/// Ce qu'on retrouve apres la relance.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    pub tabs: Vec<TabSnapshot>,
    /// Position de l'onglet actif dans `tabs`.
    pub active: usize,
}

/// Un onglet et son fil de navigation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSnapshot {
    pub history: Vec<String>,
    /// Position courante dans `history`.
    pub position: usize,
    #[serde(default)]
    pub pinned: bool,
    /// Titre connu, pour afficher l'onglet sans le charger.
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub favicon: Option<String>,
    /// Dossier d'onglets, par identifiant.
    #[serde(default)]
    pub folder: Option<String>,
    /// Profil de l'onglet (vide : profil par defaut).
    #[serde(default)]
    pub space: String,
    /// Defilement de la page au moment de la sauvegarde.
    #[serde(default)]
    pub scroll: i32,
    /// Conteneur de l'onglet (comptes a part).
    #[serde(default)]
    pub container: Option<String>,
}

impl TabSnapshot {
    /// L'adresse a rouvrir.
    pub fn current(&self) -> Option<&str> {
        self.history.get(self.position).map(String::as_str)
    }
}

fn snapshot_path(data_dir: &Path) -> PathBuf {
    data_dir.join("session.json")
}

/// Enregistre l'etat courant. Appele juste avant de relancer.
pub fn save(data_dir: &Path, snapshot: &Snapshot) {
    if let Some(parent) = snapshot_path(data_dir).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_vec_pretty(snapshot) {
        Ok(bytes) => {
            if let Err(err) = std::fs::write(snapshot_path(data_dir), bytes) {
                warn!(%err, "onglets non enregistres, ils seront perdus a la relance");
            } else {
                let pages: usize = snapshot.tabs.iter().map(|tab| tab.history.len()).sum();
                info!(
                    onglets = snapshot.tabs.len(),
                    pages,
                    "onglets enregistres pour la relance"
                );
            }
        }
        Err(err) => warn!(%err, "onglets non serialisables"),
    }
}

fn last_session_path(data_dir: &Path) -> PathBuf {
    data_dir.join("last-session.json")
}

/// Enregistre l'etat courant des onglets, pour les retrouver au prochain demarrage meme apres une fermeture.
pub fn save_last(data_dir: &Path, snapshot: &Snapshot) {
    if snapshot.tabs.is_empty() {
        return;
    }
    let path = last_session_path(data_dir);
    let written = serde_json::to_vec(snapshot).map_err(|e| e.to_string()).and_then(|bytes| {
        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, bytes).and_then(|()| std::fs::rename(&temp, &path)).map_err(|e| e.to_string())
    });
    if let Err(err) = written {
        warn!(%err, "session non enregistree");
    }
}

/// Les onglets de la derniere session, laisses tels quels pour le demarrage suivant.
pub fn load_last(data_dir: &Path) -> Option<Snapshot> {
    let text = std::fs::read_to_string(last_session_path(data_dir)).ok()?;
    match serde_json::from_str::<Snapshot>(&text) {
        Ok(snapshot) if !snapshot.tabs.is_empty() => Some(snapshot),
        Ok(_) => None,
        Err(err) => {
            warn!(%err, "derniere session illisible");
            None
        }
    }
}

/// Reprend l'etat laisse par la relance precedente, et l'oublie aussitot :
/// une session restauree ne doit pas ressurgir au demarrage suivant.
pub fn take(data_dir: &Path) -> Option<Snapshot> {
    let path = snapshot_path(data_dir);
    let text = std::fs::read_to_string(&path).ok()?;
    let _ = std::fs::remove_file(&path);
    match serde_json::from_str::<Snapshot>(&text) {
        Ok(snapshot) if !snapshot.tabs.is_empty() => {
            info!(onglets = snapshot.tabs.len(), "reprise des onglets");
            Some(snapshot)
        }
        Ok(_) => None,
        Err(err) => {
            warn!(%err, "etat de session illisible");
            None
        }
    }
}

/// Note qu'il faudra repartir une fois Chromium arrete.
pub fn request() {
    REQUESTED.store(true, Ordering::SeqCst);
}

pub fn requested() -> bool {
    REQUESTED.load(Ordering::SeqCst)
}

/// Remplace le processus courant par un neuf, avec les memes arguments.
///
/// A n'appeler qu'apres l'arret de Chromium : `exec` ne revient jamais en cas de succes,
/// et le faire plus tot laisserait ses processus enfants orphelins.
pub fn relaunch() -> std::io::Error {
    use std::os::unix::process::CommandExt;

    let program = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("echo-browser"));
    let args: Vec<String> = std::env::args().skip(1).collect();
    info!(?program, ?args, "relance du navigateur");
    std::process::Command::new(program).args(args).exec()
}
