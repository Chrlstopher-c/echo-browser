//! Responsabilite : ce que la machine garde du compte — adresse, jeton, cle de chiffrement, et pour chaque type la
//! derniere version synchronisee (base de la fusion a trois voies). Fichier lisible par l'utilisateur seul (0600).

use std::collections::BTreeMap;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::Session;
use crate::crypto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KindState {
    pub version: u64,
    pub base: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stored {
    pub email: String,
    pub token: String,
    /// Cle de chiffrement (base64). Gardee pour synchroniser sans redemander le mot de passe.
    pub encryption: String,
    #[serde(default)]
    pub kinds: BTreeMap<String, KindState>,
    #[serde(default)]
    pub last_sync: Option<i64>,
}

impl Stored {
    pub fn from_session(session: &Session) -> Self {
        Self {
            email: session.email.clone(),
            token: session.token.clone(),
            encryption: crypto::b64(&session.encryption),
            ..Default::default()
        }
    }

    pub fn session(&self) -> Option<Session> {
        let key: [u8; 32] = crypto::from_b64(&self.encryption).ok()?.try_into().ok()?;
        (!self.token.is_empty()).then(|| Session { email: self.email.clone(), token: self.token.clone(), encryption: key })
    }
}

pub fn load(path: &Path) -> Option<Stored> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

pub fn save(path: &Path, stored: &Stored) -> anyhow::Result<()> {
    let tmp = path.with_extension("tmp");
    let mut file = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&tmp)?;
    file.write_all(&serde_json::to_vec_pretty(stored)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn forget(path: &Path) {
    let _ = std::fs::remove_file(path);
}
