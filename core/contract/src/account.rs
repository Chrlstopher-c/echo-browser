//! Contrat du compte Echo : etat du compte, onglets des autres machines, contenu du coffre.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteMachineView {
    pub name: String,
    /// Derniere publication de ses onglets (secondes Unix).
    pub updated: i64,
    pub tabs: Vec<RemoteTabView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteTabView {
    pub url: String,
    pub title: String,
}

/// Le compte Echo de cette machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    /// Un service de compte est configure.
    pub available: bool,
    /// Adresse du compte connecte.
    pub email: Option<String>,
    /// Connexion ou synchronisation en cours.
    pub busy: bool,
    /// Derniere synchronisation reussie (secondes Unix).
    pub last_sync: Option<i64>,
    pub error: Option<String>,
    /// L'historique est synchronise (reglage `sync.history`).
    pub history: bool,
    /// Le compte ouvre l'administration (drapeau pose sur le serveur).
    pub admin: bool,
    /// Mode de synchronisation : `realtime`, `auto` ou `manual` (reglage `sync.mode`).
    pub mode: String,
    /// Des modifications locales attendent d'etre envoyees.
    pub pending: bool,
}

/// Un type de donnees tel que le service le garde.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultKindView {
    /// `reglages`, `favoris`, `extensions`, `onglets`, `historique`.
    pub kind: String,
    pub version: u64,
    /// Taille chiffree sur le service, en octets.
    pub bytes: u64,
    /// Derniere ecriture (millisecondes Unix).
    pub updated: i64,
    /// Nombre d'elements.
    pub count: usize,
    /// Les elements lisibles (les premiers seulement).
    pub lines: Vec<VaultLineView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultLineView {
    pub title: String,
    pub detail: String,
}
