//! Contrat d'echange entre le coeur du navigateur et son interface.
//!
//! Ce fichier et `ui/src/shared/contract.ts` sont deux miroirs du meme contrat.
//! Modifier l'un sans l'autre casse l'interface en silence.

use serde::{Deserialize, Serialize};

/// Identifiant d'onglet, partage par tous les domaines.
pub type TabId = u32;

/// Ce que l'interface demande au coeur.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UiRequest {
    NewTab { url: Option<String> },
    CloseTab { id: TabId },
    SelectTab { id: TabId },
    Navigate { id: TabId, input: String },
    GoBack { id: TabId },
    GoForward { id: TabId },
    Reload { id: TabId, bypass_cache: bool },
    Stop { id: TabId },
    ToggleShieldForSite { id: TabId },
    SetShieldEnabled { enabled: bool },
    RefreshFilterLists { force: bool },
    OpenDevTools { id: TabId },
    /// L'interface reclame une hauteur : le coeur repositionne la vue du contenu sous elle.
    SetChromeHeight { pixels: u32 },
}

/// Ce que le coeur renvoie a l'interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CoreEvent {
    TabsChanged { tabs: Vec<TabView>, active: Option<TabId> },
    TabUpdated { tab: TabView },
    ShieldUpdated { id: TabId, state: ShieldView },
    FilterListsRefreshed { count: usize },
    Notice { level: NoticeLevel, message: String },
}

/// L'etat d'un onglet tel que l'interface l'affiche.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabView {
    pub id: TabId,
    pub title: String,
    pub url: String,
    pub loading: bool,
    pub progress: f32,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub favicon: Option<String>,
}

/// L'etat du bouclier pour l'onglet courant.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldView {
    /// Bouclier actif globalement.
    pub enabled: bool,
    /// Bouclier actif sur ce site precis.
    pub active_here: bool,
    pub blocked_here: u64,
    pub blocked_total: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoticeLevel {
    Info,
    Warning,
    Error,
}
