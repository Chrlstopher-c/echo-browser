//! Contrat du panneau Reseau : ce que charge l'onglet actif, resume par domaine, et les regles du site.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkView {
    /// Page de l'onglet actif.
    pub page: String,
    pub total_bytes: u64,
    pub domains: Vec<NetDomainView>,
    /// Dernieres requetes (du domaine choisi, ou toutes), les plus recentes d'abord.
    pub requests: Vec<NetRequestView>,
    /// Domaine dont le detail est montre.
    pub focus: Option<String>,
    /// Hotes bloques par l'utilisateur sur ce site.
    pub blocked_hosts: Vec<String>,
    /// Isolement strict du site : aucune requete tierce.
    pub strict: bool,
    /// Journal d'acces du site, le plus recent d'abord.
    pub journal: Vec<JournalEntryView>,
    /// Rapport de poids : octets par type, part des tiers, requetes les plus lourdes et les plus lentes.
    pub weight: NetWeightView,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetWeightView {
    /// (type, octets), les plus lourds d'abord.
    pub by_kind: Vec<(String, u64)>,
    pub third_party_bytes: u64,
    pub heaviest: Vec<NetRequestView>,
    pub slowest: Vec<NetRequestView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryView {
    /// Secondes Unix.
    pub at: i64,
    /// `tiers`, `permission`, `telechargement`.
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetDomainView {
    pub host: String,
    pub site: String,
    pub requests: u32,
    pub bytes: u64,
    pub blocked: u32,
    pub third_party: bool,
    /// Requetes par type, les plus nombreuses d'abord : (type, nombre).
    pub kinds: Vec<(String, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetRequestView {
    pub url: String,
    pub host: String,
    pub kind: String,
    pub method: String,
    pub third_party: bool,
    /// `bouclier`, `regle` ou `isolement` quand elle a ete bloquee.
    pub blocked: Option<String>,
    pub status: Option<u16>,
    pub bytes: u64,
    pub duration_ms: Option<u64>,
}

/// Une suite de sites ouverte souvent dans le meme ordre, proposee comme routine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineProposalView {
    pub fingerprint: String,
    pub sites: Vec<String>,
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineView {
    pub id: i64,
    pub name: String,
    pub urls: Vec<String>,
}
