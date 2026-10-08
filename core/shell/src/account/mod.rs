//! Compte Echo dans le navigateur : creation, connexion, synchronisation (au demarrage, toutes les 10 min, a la
//! demande). Le travail reseau et la derivation de cles tournent hors du fil de l'interface ; les donnees locales sont
//! lues et ecrites sur ce fil (`local.rs`). Adresse du service : `ECHO_SYNC_URL`, sinon le marqueur de l'archive,
//! sinon celle fixee a la compilation (jamais dans le depot public).

mod local;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use cef::*;
use echo_account::api::Client;
use echo_account::store::{self, Stored};
use echo_contract::{AccountView, CoreEvent};
use parking_lot::Mutex;
use tracing::{info, warn};

const FIRST_SYNC_MS: i64 = 10_000;
const SYNC_EVERY_MS: i64 = 10 * 60 * 1000;

static BUSY: AtomicBool = AtomicBool::new(false);
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn service_url() -> Option<String> {
    std::env::var("ECHO_SYNC_URL").ok()
        .or_else(|| {
            let dir = crate::update::install_dir()?;
            let json: serde_json::Value = serde_json::from_slice(&std::fs::read(dir.join("release.json")).ok()?).ok()?;
            json["sync"].as_str().map(str::to_string)
        })
        .or_else(|| option_env!("ECHO_SYNC_URL").map(str::to_string))
        .filter(|url| url.starts_with("https://") || url.starts_with("http://127.0.0.1"))
}

fn stored_path() -> PathBuf {
    crate::flags::data_dir().join("compte.json")
}

pub fn publish() {
    let stored = store::load(&stored_path());
    let account = AccountView {
        available: service_url().is_some(),
        email: stored.as_ref().map(|s| s.email.clone()),
        busy: BUSY.load(Ordering::SeqCst),
        last_sync: stored.and_then(|s| s.last_sync),
        error: LAST_ERROR.lock().clone(),
    };
    crate::bridge::publish(&CoreEvent::AccountChanged { account });
}

/// Lance `job` hors du fil de l'interface (un seul a la fois), puis `done` sur le fil de l'interface.
fn background<T: Send + 'static>(job: impl FnOnce(Client) -> anyhow::Result<T> + Send + 'static, done: impl FnOnce(T) + Send + 'static) {
    let Some(url) = service_url() else { return };
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    *LAST_ERROR.lock() = None;
    publish();
    std::thread::spawn(move || {
        let result = job(Client::new(&url));
        crate::containers::later(move || {
            BUSY.store(false, Ordering::SeqCst);
            match result {
                Ok(value) => done(value),
                Err(err) => {
                    warn!(%err, "compte");
                    *LAST_ERROR.lock() = Some(err.to_string());
                }
            }
            publish();
        });
    });
}

/// Creer un compte (`create`) ou se connecter, puis synchroniser.
pub fn sign_in(email: String, password: String, create: bool) {
    let email = email.trim().to_lowercase();
    background(
        move |client| if create { client.register(&email, &password) } else { client.login(&email, &password) },
        |session| {
            if let Err(err) = store::save(&stored_path(), &Stored::from_session(&session)) {
                warn!(%err, "compte non enregistre sur la machine");
            }
            info!(email = %session.email, "compte connecte");
            crate::containers::later(sync_now);
        },
    );
}

pub fn sign_out() {
    let path = stored_path();
    let token = store::load(&path).map(|s| s.token);
    store::forget(&path);
    if let (Some(token), Some(url)) = (token, service_url()) {
        std::thread::spawn(move || {
            let _ = Client::new(&url).logout(&token);
        });
    }
    publish();
}

pub fn sync_now() {
    let Some(mut stored) = store::load(&stored_path()) else { return };
    let Some(session) = stored.session() else { return };
    let locals = local::read();
    background(
        move |client| {
            let outcome = echo_account::sync::run(&client, &session, &mut stored, &locals)?;
            stored.last_sync = Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64);
            Ok((stored, outcome))
        },
        |(stored, outcome)| {
            local::apply(&outcome.writes);
            if let Err(err) = store::save(&stored_path(), &stored) {
                warn!(%err, "etat de synchronisation non enregistre");
            }
            info!(ecrits = outcome.writes.len(), reportes = outcome.postponed.len(), "synchronisation faite");
        },
    );
}

/// Arme la synchronisation periodique (si un compte est connecte).
pub fn start() {
    schedule(FIRST_SYNC_MS);
}

fn schedule(delay_ms: i64) {
    let mut task = SyncTask::new(0);
    post_delayed_task(ThreadId::UI, Some(&mut task), delay_ms);
}

wrap_task! {
    struct SyncTask {
        unused: i32,
    }

    impl Task {
        fn execute(&self) {
            sync_now();
            schedule(SYNC_EVERY_MS);
        }
    }
}
