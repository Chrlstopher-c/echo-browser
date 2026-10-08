//! Responsabilite : une passe de synchronisation. Entree : les valeurs locales de chaque type ; sortie : celles a
//! ecrire localement. Entre les deux, le coffre est lu, fusionne a trois voies et reecrit si besoin. Bloquant : hors du
//! fil de l'interface.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::api::{Client, Put, Session};
use crate::merge::{self, Shape};
use crate::store::{KindState, Stored};
use crate::crypto;

/// Ce qui se synchronise, et comment le fusionner.
/// `onglets` : une entree par machine (ses onglets ouverts) ; chacune ne modifie que la sienne.
pub const KINDS: [(&str, Shape); 5] = [
    ("reglages", Shape::Map),
    ("favoris", Shape::KeyedList("url")),
    ("extensions", Shape::SetMap),
    ("onglets", Shape::Map),
    ("historique", Shape::Latest { max: HISTORY_MAX }),
];

/// Adresses d'historique gardees dans le coffre (les plus recentes).
pub const HISTORY_MAX: usize = 1000;

pub struct Outcome {
    /// Valeurs a ecrire localement (seulement celles qui changent).
    pub writes: BTreeMap<String, Value>,
    /// Types reportes a la passe suivante (une autre machine ecrivait en meme temps).
    pub postponed: Vec<String>,
}

pub fn run(client: &Client, session: &Session, stored: &mut Stored, locals: &BTreeMap<String, Value>) -> anyhow::Result<Outcome> {
    let items = client.vault(&session.token)?;
    let mut outcome = Outcome { writes: BTreeMap::new(), postponed: Vec::new() };
    for (kind, shape) in KINDS {
        let Some(local) = locals.get(kind) else { continue };
        let remote = match items.iter().find(|i| i.kind == kind) {
            Some(item) => {
                let plain = crypto::open(&session.encryption, kind, &item.data)?;
                Some((item.version, serde_json::from_slice::<Value>(&plain)?))
            }
            None => None,
        };
        sync_kind(client, session, stored, &mut outcome, (kind, shape), local, remote)?;
    }
    Ok(outcome)
}

/// Un type : fusion, ecriture locale si elle change, envoi au service si lui change.
fn sync_kind(
    client: &Client,
    session: &Session,
    stored: &mut Stored,
    outcome: &mut Outcome,
    (kind, shape): (&str, Shape),
    local: &Value,
    remote: Option<(u64, Value)>,
) -> anyhow::Result<()> {
    let base = stored.kinds.get(kind).map(|s| s.base.clone()).unwrap_or_else(|| merge::empty(shape));
    let remote_value = remote.as_ref().map(|(_, v)| v.clone()).unwrap_or_else(|| merge::empty(shape));
    let merged = merge::merge(shape, &base, local, &remote_value);
    if &merged != local {
        outcome.writes.insert(kind.to_string(), merged.clone());
    }
    let remote_version = remote.as_ref().map_or(0, |(v, _)| *v);
    if remote.is_some() && merged == remote_value {
        stored.kinds.insert(kind.to_string(), KindState { version: remote_version, base: merged });
        return Ok(());
    }
    let sealed = crypto::seal(&session.encryption, kind, &serde_json::to_vec(&merged)?)?;
    match client.put(&session.token, kind, remote_version, &sealed)? {
        Put::Saved(version) => {
            stored.kinds.insert(kind.to_string(), KindState { version, base: merged });
        }
        Put::Conflict(..) => outcome.postponed.push(kind.to_string()),
    }
    Ok(())
}
