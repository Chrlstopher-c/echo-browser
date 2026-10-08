//! Compte Echo dans le navigateur : creation, connexion, synchronisation (au demarrage, toutes les 10 min, a la
//! demande). Le travail reseau et la derivation de cles tournent hors du fil de l'interface ; les donnees locales sont
//! lues et ecrites sur ce fil (`local.rs`). Adresse du service : `ECHO_SYNC_URL`, sinon le marqueur de l'archive,
//! sinon celle fixee a la compilation (jamais dans le depot public).

mod admin;
mod history;
mod local;
mod machine;
mod vault;

pub(crate) mod schedule;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use echo_account::api::Client;
use echo_account::store::{self, Stored};
use echo_contract::{AccountView, CoreEvent};
use parking_lot::Mutex;
use tracing::{info, warn};


static BUSY: AtomicBool = AtomicBool::new(false);
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

pub(crate) fn service_url() -> Option<String> {
    std::env::var("ECHO_SYNC_URL").ok()
        .or_else(|| {
            let dir = crate::update::install_dir()?;
            let json: serde_json::Value = serde_json::from_slice(&std::fs::read(dir.join("release.json")).ok()?).ok()?;
            json["sync"].as_str().map(str::to_string)
        })
        .or_else(|| option_env!("ECHO_SYNC_URL").map(str::to_string))
        .filter(|url| url.starts_with("https://") || url.starts_with("http://127.0.0.1"))
}

pub(super) fn stored_path() -> PathBuf {
    crate::flags::data_dir().join("compte.json")
}

pub fn publish() {
    let stored = store::load(&stored_path());
    machine::publish(stored.as_ref().and_then(|s| s.kinds.get("onglets")).map(|k| &k.base));
    let account = AccountView {
        available: service_url().is_some(),
        email: stored.as_ref().map(|s| s.email.clone()),
        busy: BUSY.load(Ordering::SeqCst),
        last_sync: stored.as_ref().and_then(|s| s.last_sync),
        error: LAST_ERROR.lock().clone(),
        history: history::enabled(),
        admin: stored.as_ref().is_some_and(|s| s.admin),
        mode: schedule::mode().as_str().to_string(),
        pending: schedule::pending(),
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
            crate::containers::later(|| { sync_now(); });
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

/// Types dont l'application locale peut ecraser une modification faite pendant la synchro : verifies avant d'ecrire.
const GUARDED: [&str; 3] = ["reglages", "favoris", "extensions"];

/// Lance une synchronisation ; rend faux si une autre tourne deja (ou pas de compte). Ne perd rien : une modification
/// locale faite pendant l'aller-retour n'est pas ecrasee (ce type est garde tel quel et repart a la passe suivante).
pub fn sync_now() -> bool {
    let Some(mut stored) = store::load(&stored_path()) else { return false };
    let Some(session) = stored.session() else { return false };
    if BUSY.load(Ordering::SeqCst) {
        return false;
    }
    let mut locals = local::read();
    locals.insert("onglets".to_string(), machine::local_value(stored.kinds.get("onglets").map(|k| &k.base)));
    if history::enabled() {
        locals.insert("historique".to_string(), history::local_value());
    }
    schedule::snapshot_taken();
    let before = stored.kinds.clone();
    let sent: std::collections::BTreeMap<String, serde_json::Value> =
        GUARDED.iter().filter_map(|k| Some((k.to_string(), locals.get(*k)?.clone()))).collect();
    background(
        move |client| {
            Ok(echo_account::sync::run(&client, &session, &mut stored, &locals).map(|outcome| {
                stored.admin = client.state(&session.token).map(|(_, admin)| admin).unwrap_or(stored.admin);
                stored.last_sync = Some(echo_library::now());
                (stored, outcome)
            }))
        },
        move |result| match result {
            Ok((stored, outcome)) => finish_sync(stored, outcome, &before, &sent),
            Err(err) => {
                warn!(%err, "synchronisation");
                *LAST_ERROR.lock() = Some(err.to_string());
                schedule::sync_failed();
            }
        },
    );
    true
}

fn finish_sync(
    mut stored: Stored,
    mut outcome: echo_account::sync::Outcome,
    before: &std::collections::BTreeMap<String, echo_account::store::KindState>,
    sent: &std::collections::BTreeMap<String, serde_json::Value>,
) {
    let now = local::read();
    let mut again = !outcome.postponed.is_empty();
    for kind in GUARDED {
        if outcome.writes.contains_key(kind) && now.get(kind) != sent.get(kind) {
            // Modifie ici pendant la synchro : on n'ecrase pas, et la base reste l'ancienne pour la prochaine fusion.
            outcome.writes.remove(kind);
            match before.get(kind) {
                Some(state) => stored.kinds.insert(kind.to_string(), state.clone()),
                None => stored.kinds.remove(kind),
            };
            again = true;
        }
    }
    local::apply(&outcome.writes);
    if let Some(merged) = outcome.writes.get("historique") {
        history::apply(merged);
    }
    if let Err(err) = store::save(&stored_path(), &stored) {
        warn!(%err, "etat de synchronisation non enregistre");
    }
    info!(ecrits = outcome.writes.len(), reportes = outcome.postponed.len(), again, "synchronisation faite");
    schedule::sync_done(again);
}

pub use admin::{delete_account as admin_delete, detail as admin_detail, refresh as admin_refresh,
    set_flag as admin_set_flag, sign_out_account as admin_sign_out};

/// L'interface a change un reglage : celui de l'historique se reflete dans l'etat du compte, et part tout de suite.
pub fn setting_changed(key: &str) {
    if key == history::SETTING || key == schedule::MODE_SETTING {
        publish();
        schedule::mode_changed();
    }
}

/// Lit le coffre sur le service et le montre dechiffre (evenement `AccountVault`).
pub fn inspect() {
    let Some(session) = store::load(&stored_path()).and_then(|s| s.session()) else { return };
    background(
        move |client| Ok(vault::view(&session, &client.vault(&session.token)?)),
        |kinds| crate::bridge::publish(&CoreEvent::AccountVault { kinds }),
    );
}

/// Supprime le compte et tout ce que le service en garde, puis oublie le compte sur cette machine. Les donnees locales
/// (favoris, historique…) restent.
pub fn delete() {
    let Some(stored) = store::load(&stored_path()) else { return };
    let token = stored.token.clone();
    background(
        move |client| client.delete_account(&token),
        move |()| {
            store::forget(&stored_path());
            info!(email = %stored.email, "compte supprime du service");
            crate::bridge::publish(&CoreEvent::AccountVault { kinds: Vec::new() });
        },
    );
}

/// Synchronisation automatique : au lancement, apres chaque lot de modifications, et controle periodique du service.
pub fn start() {
    schedule::start();
}
