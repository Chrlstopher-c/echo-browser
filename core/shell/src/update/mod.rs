//! Mise a jour automatique de la version installee (archive des releases) : verification de la derniere release,
//! preparation en arriere-plan, bascule au redemarrage. Jamais pour la version de developpement (pas de `release.json`
//! a cote de l'executable).

mod apply;
mod github;
mod stage;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use cef::*;
use echo_contract::{CoreEvent, UpdateStatus, UpdateView};
use parking_lot::Mutex;
use tracing::{info, warn};

pub use apply::{confirm_started, launcher};

const MARKER: &str = "release.json";
const AUTO_SETTING: &str = "updates.auto";
const FIRST_CHECK_S: i64 = 30;
const CHECK_EVERY_S: i64 = 6 * 3600;

static STATE: Mutex<Option<(UpdateStatus, Option<String>, Option<String>)>> = Mutex::new(None);
static BUSY: AtomicBool = AtomicBool::new(false);

/// Le dossier de l'archive installee, s'il y en a une (le marqueur est pose par `tools/package-release.sh`).
pub fn install_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.to_path_buf();
    dir.join(MARKER).is_file().then_some(dir)
}

/// La version annoncee par le marqueur d'une archive.
pub fn read_release_version(dir: &Path) -> Option<String> {
    let raw = std::fs::read(dir.join(MARKER)).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&raw).ok()?;
    json["version"].as_str().map(str::to_string)
}

fn repo() -> Option<String> {
    let raw = std::fs::read(install_dir()?.join(MARKER)).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&raw).ok()?;
    json["repo"].as_str().map(str::to_string)
}

/// La version installee : celle de l'archive (son marqueur), sinon celle du programme.
fn current() -> String {
    install_dir().and_then(|dir| read_release_version(&dir)).unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
}

fn set(status: UpdateStatus, latest: Option<String>, error: Option<String>) {
    *STATE.lock() = Some((status, latest, error));
    publish();
}

pub fn publish() {
    let (status, latest, error) = match (install_dir(), STATE.lock().clone()) {
        (None, _) => (UpdateStatus::Unavailable, None, None),
        (Some(_), Some(state)) => state,
        (Some(_), None) => (UpdateStatus::UpToDate, None, None),
    };
    let update = UpdateView { current: current(), status, latest, error };
    crate::bridge::publish(&CoreEvent::UpdateChanged { update });
}

fn auto_enabled() -> bool {
    !crate::session::with(|s| {
        echo_library::settings::all(&s.library)
            .into_iter()
            .any(|(key, value)| key == AUTO_SETTING && value == echo_library::settings::Value::Flag(false))
    })
    .unwrap_or(false)
}

/// Arme les verifications : peu apres le demarrage, puis toutes les 6 h (`ECHO_UPDATE_DELAY_S` pour les essais).
pub fn start() {
    confirm_started();
    if install_dir().is_none() {
        return;
    }
    let first = std::env::var("ECHO_UPDATE_DELAY_S").ok().and_then(|v| v.parse().ok()).unwrap_or(FIRST_CHECK_S);
    schedule(first * 1000);
}

fn schedule(delay_ms: i64) {
    let mut task = CheckTask::new(0);
    post_delayed_task(ThreadId::UI, Some(&mut task), delay_ms);
}

wrap_task! {
    struct CheckTask {
        unused: i32,
    }

    impl Task {
        fn execute(&self) {
            if auto_enabled() {
                check(true);
            }
            schedule(CHECK_EVERY_S * 1000);
        }
    }
}

/// Verifie la derniere release ; `prepare` : la telecharger et la preparer si elle est plus recente.
pub fn check(prepare: bool) {
    let (Some(install), Some(repo)) = (install_dir(), repo()) else { return };
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    if !matches!(STATE.lock().as_ref().map(|s| s.0), Some(UpdateStatus::Ready)) {
        set(UpdateStatus::Checking, None, None);
    }
    std::thread::spawn(move || {
        let outcome = run(&install, &repo, prepare);
        crate::containers::later(move || {
            BUSY.store(false, Ordering::SeqCst);
            match outcome {
                Ok((status, latest)) => set(status, latest, None),
                Err(err) => {
                    warn!(%err, "mise a jour");
                    set(UpdateStatus::Failed, None, Some(err));
                }
            }
        });
    });
}

fn run(install: &Path, repo: &str, prepare: bool) -> Result<(UpdateStatus, Option<String>), String> {
    let release = github::latest(repo)?;
    if !github::is_newer(&release.version, &current()) {
        return Ok((UpdateStatus::UpToDate, Some(release.version)));
    }
    let staged = stage::staged_dir(install);
    if read_release_version(&staged).as_deref() == Some(release.version.as_str()) {
        return Ok((UpdateStatus::Ready, Some(release.version)));
    }
    if !prepare {
        return Ok((UpdateStatus::Available, Some(release.version)));
    }
    let version = release.version.clone();
    crate::containers::later(move || set(UpdateStatus::Downloading, Some(version), None));
    stage::prepare(install, &release)?;
    info!(version = %release.version, "mise a jour prete, appliquee au prochain demarrage");
    Ok((UpdateStatus::Ready, Some(release.version)))
}
