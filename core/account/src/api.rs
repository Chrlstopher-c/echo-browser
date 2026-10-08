//! Responsabilite : parler au service de compte (Worker). Inscription et connexion derivent les cles ici ; seule la cle
//! d'acces part sur le reseau. Le coffre ne voit passer que des donnees deja chiffrees.

use serde_json::{json, Value};

use crate::crypto::{self, Keys};

const TIMEOUT_S: u64 = 20;

/// Une session ouverte : le jeton pour le service, la cle de chiffrement pour les donnees.
#[derive(Clone)]
pub struct Session {
    pub email: String,
    pub token: String,
    pub encryption: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct Item {
    pub kind: String,
    pub version: u64,
    pub data: String,
    /// Derniere ecriture sur le service (millisecondes Unix).
    pub updated: i64,
}

pub enum Put {
    Saved(u64),
    /// Une autre machine a ecrit entre-temps : sa version et ses donnees.
    Conflict(u64, Option<String>),
}

pub struct Client {
    base: String,
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(TIMEOUT_S)))
        .http_status_as_error(false)
        .build()
        .into()
}

fn error_of(status: u16, body: &Value) -> anyhow::Error {
    let message = body["erreur"].as_str().unwrap_or("erreur du service");
    anyhow::anyhow!("{message} ({status})")
}

impl Client {
    pub fn new(base: &str) -> Self {
        Self { base: base.trim_end_matches('/').to_string() }
    }

    fn call(&self, method: &str, path: &str, token: Option<&str>, body: Option<Value>) -> anyhow::Result<(u16, Value)> {
        let url = format!("{}{path}", self.base);
        let agent = agent();
        let auth = token.map(|t| format!("Bearer {t}"));
        let version = env!("CARGO_PKG_VERSION");
        let mut response = match (method, body) {
            ("GET", _) => {
                let mut request = agent.get(&url).header("X-Echo-Version", version);
                if let Some(auth) = &auth {
                    request = request.header("Authorization", auth);
                }
                request.call()?
            }
            ("DELETE", _) => {
                let mut request = agent.delete(&url).header("X-Echo-Version", version);
                if let Some(auth) = &auth {
                    request = request.header("Authorization", auth);
                }
                request.call()?
            }
            (method, body) => {
                let mut request = if method == "PUT" { agent.put(&url) } else { agent.post(&url) }
                    .header("X-Echo-Version", version);
                if let Some(auth) = &auth {
                    request = request.header("Authorization", auth);
                }
                request.send_json(body.unwrap_or(json!({})))?
            }
        };
        let status = response.status().as_u16();
        let text = response.body_mut().read_to_string().unwrap_or_default();
        Ok((status, serde_json::from_str(&text).unwrap_or(Value::Null)))
    }

    fn salt(&self, email: &str) -> anyhow::Result<(Vec<u8>, u32)> {
        let (status, body) = self.call("POST", "/v1/sel", None, Some(json!({ "email": email })))?;
        anyhow::ensure!(status == 200, error_of(status, &body));
        let salt = crypto::from_b64(body["sel"].as_str().unwrap_or_default())?;
        let iterations = body["iterations"].as_u64().unwrap_or(u64::from(crypto::ITERATIONS)) as u32;
        Ok((salt, iterations))
    }

    pub fn register(&self, email: &str, password: &str) -> anyhow::Result<Session> {
        let salt = crypto::random_bytes::<{ crypto::SALT_LEN }>()?;
        let Keys { access, encryption } = crypto::derive(password, &salt, crypto::ITERATIONS);
        let body = json!({
            "email": email, "cleAcces": crypto::b64(&access), "sel": crypto::b64(&salt), "iterations": crypto::ITERATIONS,
        });
        let (status, body) = self.call("POST", "/v1/inscription", None, Some(body))?;
        anyhow::ensure!(status == 201, error_of(status, &body));
        Ok(Session { email: email.to_string(), token: body["jeton"].as_str().unwrap_or_default().to_string(), encryption })
    }

    pub fn login(&self, email: &str, password: &str) -> anyhow::Result<Session> {
        let (salt, iterations) = self.salt(email)?;
        let Keys { access, encryption } = crypto::derive(password, &salt, iterations);
        let body = json!({ "email": email, "cleAcces": crypto::b64(&access) });
        let (status, body) = self.call("POST", "/v1/connexion", None, Some(body))?;
        anyhow::ensure!(status == 200, error_of(status, &body));
        Ok(Session { email: email.to_string(), token: body["jeton"].as_str().unwrap_or_default().to_string(), encryption })
    }

    pub fn logout(&self, token: &str) -> anyhow::Result<()> {
        self.call("POST", "/v1/deconnexion", Some(token), None).map(drop)
    }

    /// Supprime le compte et tout son coffre sur le service. Definitif.
    pub fn delete_account(&self, token: &str) -> anyhow::Result<()> {
        let (status, body) = self.call("DELETE", "/v1/compte", Some(token), None)?;
        anyhow::ensure!(status == 204, error_of(status, &body));
        Ok(())
    }

    pub fn vault(&self, token: &str) -> anyhow::Result<Vec<Item>> {
        let (status, body) = self.call("GET", "/v1/coffre", Some(token), None)?;
        anyhow::ensure!(status == 200, error_of(status, &body));
        Ok(body["elements"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .map(|i| Item {
                        kind: i["type"].as_str().unwrap_or_default().to_string(),
                        version: i["version"].as_u64().unwrap_or(0),
                        data: i["donnees"].as_str().unwrap_or_default().to_string(),
                        updated: i["maj_le"].as_i64().unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Le compte ouvre-t-il l'administration ?
    pub fn is_admin(&self, token: &str) -> anyhow::Result<bool> {
        let (status, body) = self.call("GET", "/v1/moi", Some(token), None)?;
        anyhow::ensure!(status == 200, error_of(status, &body));
        Ok(body["admin"].as_bool().unwrap_or(false))
    }

    /// Appel du tableau de bord avec la session (le service verifie que le compte est administrateur).
    pub fn admin(&self, token: &str, method: &str, path: &str, body: Option<Value>) -> anyhow::Result<Value> {
        anyhow::ensure!(path.starts_with("/v1/admin/"), "chemin d'administration attendu");
        let (status, body) = self.call(method, path, Some(token), body)?;
        anyhow::ensure!((200..300).contains(&status), error_of(status, &body));
        Ok(body)
    }

    pub fn put(&self, token: &str, kind: &str, base: u64, data: &str) -> anyhow::Result<Put> {
        let body = json!({ "base": base, "donnees": data });
        let (status, body) = self.call("PUT", &format!("/v1/coffre/{kind}"), Some(token), Some(body))?;
        match status {
            200 => Ok(Put::Saved(body["version"].as_u64().unwrap_or(base + 1))),
            409 => Ok(Put::Conflict(body["version"].as_u64().unwrap_or(0), body["donnees"].as_str().map(str::to_string))),
            _ => Err(error_of(status, &body)),
        }
    }
}
