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
                let from = CefString::from(&item.original_url()).to_string();
                crate::network::journal(&from, "telechargement", &name);
                record(item, &name);
            }
            if let Some(callback) = callback {
                let target = download_dir().join(&name);
                if ask_location() {
                    ask_then_continue(callback.clone(), target);
                } else {
                    callback.cont(Some(&CefString::from(target.to_string_lossy().as_ref())), 0);
                }
            }
            info!(%name, "telechargement demarre");
            notice(format!("Téléchargement de {name}…"));
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

/// Reglage « Demander où enregistrer ».
fn ask_location() -> bool {
    use echo_library::settings::Value;
    let value = crate::session::with(|s| echo_library::settings::get(&s.library, "downloads.ask_location")).flatten();
    matches!(value, Some(Value::Flag(true)))
}

thread_local! {
    /// Telechargements en attente du selecteur, rendus au thread interface par numero.
    static WAITING: std::cell::RefCell<Vec<(u64, BeforeDownloadCallback)>> = const { std::cell::RefCell::new(Vec::new()) };
}
static NEXT_WAIT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Le selecteur du portail (celui du bureau ; la boite de CEF faisait tomber Echo) choisit le fichier ;
/// Chromium attend la reponse. Annule si l'utilisateur ferme le selecteur.
fn ask_then_continue(callback: BeforeDownloadCallback, target: std::path::PathBuf) {
    let wait = NEXT_WAIT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    WAITING.with(|w| w.borrow_mut().push((wait, callback)));
    std::thread::spawn(move || {
        let mut dialog = rfd::FileDialog::new().set_title("Enregistrer le fichier");
        if let Some(dir) = target.parent() {
            dialog = dialog.set_directory(dir);
        }
        if let Some(name) = target.file_name() {
            dialog = dialog.set_file_name(name.to_string_lossy());
        }
        let chosen = dialog.save_file();
        crate::containers::later(move || {
            let Some(callback) = WAITING.with(|w| {
                let mut w = w.borrow_mut();
                w.iter().position(|(id, _)| *id == wait).map(|at| w.remove(at).1)
            }) else { return };
            match chosen {
                Some(path) => callback.cont(Some(&CefString::from(path.to_string_lossy().as_ref())), 0),
                None => info!("enregistrement annule"),
            }
        });
    });
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

    let finished = download.state == State::Complete && ANNOUNCED.lock().insert(download.id);
    let name = download.file_name.clone();
    crate::session::with(|s| downloads::upsert(&s.library, &download));
    crate::bridge::library::publish_downloads();
    if finished {
        let id = download.id;
        crate::bridge::publish(&echo_contract::CoreEvent::Notice {
            level: echo_contract::NoticeLevel::Info,
            message: format!("Téléchargé : {name}"),
            actions: vec![
                echo_contract::NoticeAction { label: "Ouvrir".into(), request: echo_contract::UiRequest::OpenDownload { id } },
                echo_contract::NoticeAction {
                    label: "Afficher dans le dossier".into(),
                    request: echo_contract::UiRequest::RevealDownload { id },
                },
            ],
        });
    }
}

/// Telechargements dont la fin a deja ete annoncee : Chromium rappelle plusieurs fois l'etat final.
static ANNOUNCED: std::sync::LazyLock<Mutex<std::collections::HashSet<u32>>> =
    std::sync::LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

fn notice(message: String) {
    crate::bridge::publish(&echo_contract::CoreEvent::notice(echo_contract::NoticeLevel::Info, message));
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
