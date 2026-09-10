//! Responsabilite : suivre les telechargements de Chromium et les rendre a la bibliotheque.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use echo_library::downloads::{self, Download, State};
use parking_lot::Mutex;
use std::collections::HashMap;
use tracing::info;

/// Ordres en attente pour les telechargements en cours : annuler, mettre en pause.
/// Renseignes depuis l'interface, consommes a la prochaine mise a jour de Chromium.
static PENDING_CANCELS: Mutex<Option<HashMap<u32, ()>>> = Mutex::new(None);

/// Demande l'annulation d'un telechargement. Elle prend effet a sa prochaine mise a jour.
pub fn request_cancel(id: u32) {
    PENDING_CANCELS.lock().get_or_insert_with(HashMap::new).insert(id, ());
}

fn take_cancel(id: u32) -> bool {
    PENDING_CANCELS.lock().as_mut().is_some_and(|map| map.remove(&id).is_some())
}

wrap_download_handler! {
    pub struct Transfers {
        marker: (),
    }

    impl DownloadHandler {
        /// Sans cette autorisation, tout telechargement est refuse avant meme d'etre
        /// annonce : la valeur par defaut de la liaison vaut « non ».
        fn can_download(
            &self,
            _browser: Option<&mut Browser>,
            _url: Option<&CefString>,
            _request_method: Option<&CefString>,
        ) -> i32 {
            1
        }

        /// Chromium demande ou ecrire. On accepte son choix sans boite de dialogue.
        fn on_before_download(
            &self,
            _browser: Option<&mut Browser>,
            item: Option<&mut DownloadItem>,
            suggested_name: Option<&CefString>,
            callback: Option<&mut BeforeDownloadCallback>,
        ) -> i32 {
            let name = suggested_name.map(CefString::to_string).unwrap_or_default();
            if let Some(item) = item {
                record(item, &name);
            }
            if let Some(callback) = callback {
                let target = download_dir().join(&name);
                callback.cont(Some(&CefString::from(target.to_string_lossy().as_ref())), 0);
            }
            info!(%name, "telechargement demarre");
            1
        }

        fn on_download_updated(
            &self,
            _browser: Option<&mut Browser>,
            item: Option<&mut DownloadItem>,
            callback: Option<&mut DownloadItemCallback>,
        ) {
            let Some(item) = item else { return };
            if take_cancel(item.id()) {
                if let Some(callback) = callback {
                    callback.cancel();
                }
            }
            record(item, "");
        }
    }
}

/// Dossier de destination : celui du bureau s'il est declare, sinon le classique.
fn download_dir() -> std::path::PathBuf {
    if let Some(dir) = std::env::var_os("XDG_DOWNLOAD_DIR") {
        return std::path::PathBuf::from(dir);
    }
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from).unwrap_or_default();
    for name in ["Telechargements", "Downloads"] {
        let candidate = home.join(name);
        if candidate.is_dir() {
            return candidate;
        }
    }
    home.join("Downloads")
}

/// Ecrit l'etat courant du telechargement dans la bibliotheque et previent l'interface.
fn record(item: &DownloadItem, fallback_name: &str) {
    let path = CefString::from(&item.full_path()).to_string();
    // Le nom suggere n'est renseigne qu'a l'annonce : ensuite, c'est le chemin qui fait foi.
    let from_path = std::path::Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    let suggested = CefString::from(&item.suggested_file_name()).to_string();
    let file_name = [from_path.as_str(), suggested.as_str(), fallback_name, "fichier"]
        .into_iter()
        .find(|candidate| !candidate.is_empty())
        .unwrap_or("fichier")
        .to_string();

    let total = item.total_bytes();
    let download = Download {
        id: item.id(),
        file_name,
        url: CefString::from(&item.url()).to_string(),
        path: (!path.is_empty()).then_some(path),
        received: item.received_bytes().max(0) as u64,
        total: (total > 0).then_some(total as u64),
        state: state_of(item),
        started_at: echo_library::now(),
    };

    crate::session::with(|s| downloads::upsert(&s.library, &download));
    crate::bridge::library::publish_downloads();
}

fn state_of(item: &DownloadItem) -> State {
    if item.is_complete() == 1 {
        State::Complete
    } else if item.is_canceled() == 1 {
        State::Cancelled
    } else if item.is_interrupted() == 1 {
        State::Failed
    } else if item.is_in_progress() == 1 {
        State::Running
    } else {
        State::Paused
    }
}
