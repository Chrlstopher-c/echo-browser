//! Contrat de la bibliotheque : favoris, historique, telechargements, reglages, permissions retenues, avis.

use serde::{Deserialize, Serialize};

use crate::DownloadId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkView {
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub added_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntryView {
    pub url: String,
    pub title: String,
    pub favicon: Option<String>,
    pub visited_at: i64,
    /// Nombre de visites sur cette adresse.
    pub visits: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadView {
    pub id: DownloadId,
    pub file_name: String,
    pub url: String,
    /// Chemin complet, une fois le fichier ecrit.
    pub path: Option<String>,
    pub received: u64,
    pub total: Option<u64>,
    pub state: DownloadState,
    pub started_at: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DownloadState {
    Running,
    Paused,
    Complete,
    Cancelled,
    Failed,
}

/// Un reglage et sa valeur courante.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingView {
    pub key: String,
    pub value: SettingValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum SettingValue {
    Flag(bool),
    Text(String),
    Number(f64),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoticeLevel {
    Info,
    Warning,
    Error,
}


/// Une decision de permission retenue pour un site.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionGrantView {
    pub origin: String,
    pub kind: String,
    pub allow: bool,
}
