//! Section Administration : le tableau de bord du service, lu et pilote avec la session du compte connecte. Le
//! service n'accepte que les comptes administrateurs ; ici, rien n'est garde (chaque ouverture relit le service).

use echo_account::api::Client;
use echo_account::store;
use echo_contract::CoreEvent;
use parking_lot::Mutex;
use serde_json::{json, Value};
use tracing::warn;

/// Lance `job` avec un client et le jeton du compte, hors du fil de l'interface, puis rediffuse le tableau de bord.
fn with_session(job: impl FnOnce(&Client, &str) -> anyhow::Result<()> + Send + 'static, query: String) {
    let Some(url) = super::service_url() else { return };
    let Some(token) = store::load(&super::stored_path()).map(|s| s.token) else { return };
    std::thread::spawn(move || {
        let client = Client::new(&url);
        let action = job(&client, &token);
        let read = action.and_then(|()| {
            let summary = client.admin(&token, "GET", "/v1/admin/resume", None)?;
            let path = format!("/v1/admin/comptes?q={}", encode(&query));
            let accounts = client.admin(&token, "GET", &path, None)?["comptes"].clone();
            let signals = client.admin(&token, "GET", "/v1/admin/signaux", None).unwrap_or(Value::Null);
            Ok((summary, accounts, signals))
        });
        let event = match read {
            Ok((summary, accounts, signals)) => CoreEvent::AdminData { summary, accounts, signals, error: None },
            Err(err) => {
                warn!(%err, "administration");
                let error = Some(err.to_string());
                CoreEvent::AdminData { summary: Value::Null, accounts: Value::Null, signals: Value::Null, error }
            }
        };
        crate::containers::later(move || crate::bridge::publish(&event));
    });
}

fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn valid_id(id: &str) -> bool {
    id.len() == 36 && id.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-')
}

/// Derniere recherche de l'interface : les actions rediffusent la liste filtree de la meme facon.
static QUERY: Mutex<String> = Mutex::new(String::new());

pub fn refresh(query: String) {
    *QUERY.lock() = query.clone();
    with_session(|_, _| Ok(()), query);
}

fn act(id: &str, method: &'static str, suffix: &str, body: Option<Value>) {
    if !valid_id(id) {
        return;
    }
    let path = format!("/v1/admin/comptes/{id}{suffix}");
    with_session(move |client, token| client.admin(token, method, &path, body).map(drop), QUERY.lock().clone());
}

/// Fiche d'un compte (lecture seule), diffusee a part du tableau de bord.
pub fn detail(id: &str) {
    let (Some(url), Some(token)) = (super::service_url(), store::load(&super::stored_path()).map(|s| s.token)) else {
        return;
    };
    if !valid_id(id) {
        return;
    }
    let path = format!("/v1/admin/comptes/{id}");
    std::thread::spawn(move || {
        let event = match Client::new(&url).admin(&token, "GET", &path, None) {
            Ok(detail) => CoreEvent::AdminAccount { detail, error: None },
            Err(err) => CoreEvent::AdminAccount { detail: Value::Null, error: Some(err.to_string()) },
        };
        crate::containers::later(move || crate::bridge::publish(&event));
    });
}

pub fn sign_out_account(id: &str) {
    act(id, "POST", "/deconnexion", None);
}

pub fn delete_account(id: &str) {
    act(id, "DELETE", "", None);
}

pub fn set_flag(id: &str, admin: bool) {
    act(id, "POST", "/admin", Some(json!({ "admin": admin })));
}
