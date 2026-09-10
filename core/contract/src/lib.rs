//! Contrat d'echange entre le coeur du navigateur et son interface.
//!
//! Ce fichier et `ui/src/shared/contract.ts` sont deux miroirs du meme contrat.
//! Modifier l'un sans l'autre casse l'interface en silence.

use serde::{Deserialize, Serialize};

pub type TabId = u32;
pub type DownloadId = u32;

/// Ce que l'interface demande au coeur.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UiRequest {
    // --- Onglets et navigation ---
    NewTab { url: Option<String> },
    CloseTab { id: TabId },
    SelectTab { id: TabId },
    /// Deplace un onglet a une nouvelle position dans la liste.
    MoveTab { id: TabId, to: usize },
    PinTab { id: TabId, pinned: bool },
    Navigate { id: TabId, input: String },
    GoBack { id: TabId },
    GoForward { id: TabId },
    Reload { id: TabId, bypass_cache: bool },
    Stop { id: TabId },
    /// Facteur de zoom de la page, 1.0 etant la taille naturelle.
    SetZoom { id: TabId, factor: f32 },
    OpenDevTools { id: TabId },
    /// Sort du plein ecran, quand l'utilisateur le demande depuis l'interface.
    ExitFullscreen,

    // --- Mise en page ---
    /// L'interface reclame une largeur : le coeur repositionne la vue du contenu a sa droite.
    SetChromeWidth { pixels: u32 },
    SetSidebarCollapsed { collapsed: bool },
    /// Teinte dominante de l'espace courant, appliquee au cadre autour de la page.
    SetAccent { color: String },

    // --- Bouclier ---
    ToggleShieldForSite { id: TabId },
    SetShieldEnabled { enabled: bool },
    RefreshFilterLists { force: bool },
    /// Active ou desactive une liste de filtres.
    SetFilterListEnabled { id: String, enabled: bool },

    // --- Extensions ---
    /// Ouvre la fiche d'une extension, ou le catalogue, pour que Chromium l'installe.
    InstallExtension { source: String },
    RemoveExtension { id: String },
    SetExtensionEnabled { id: String, enabled: bool },
    /// Ouvre le gestionnaire d'extensions de Chromium.
    OpenExtensionManager,
    /// Ouvre la fenetre d'une extension, ancree sous son icone.
    OpenExtensionPopup { id: String, anchor: AnchorRect },
    /// Referme la fenetre d'extension ouverte, s'il y en a une.
    CloseExtensionPopup,
    /// Ouvre la page de reglages d'une extension dans un onglet.
    OpenExtensionOptions { id: String },

    // --- Bibliotheque ---
    AddBookmark { id: TabId },
    RemoveBookmark { url: String },
    /// Deplace un favori dans la liste.
    MoveBookmark { url: String, to: usize },
    RemoveHistoryEntry { url: String, visited_at: i64 },
    ClearHistory,
    /// Filtre l'historique. Une requete vide rend les entrees les plus recentes.
    SearchHistory { terms: String },

    // --- Telechargements ---
    OpenDownload { id: DownloadId },
    /// Ouvre le dossier contenant le fichier.
    RevealDownload { id: DownloadId },
    CancelDownload { id: DownloadId },
    /// Retire l'entree de la liste, sans effacer le fichier.
    ForgetDownload { id: DownloadId },

    // --- Reglages ---
    UpdateSetting { key: String, value: SettingValue },

    // --- Cycle de vie ---
    /// Relance le navigateur. Les onglets ouverts sont retrouves apres la relance.
    RestartBrowser,
}

/// Ce que le coeur renvoie a l'interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum CoreEvent {
    TabsChanged { tabs: Vec<TabView>, active: Option<TabId> },
    TabUpdated { tab: TabView },
    ShieldUpdated { id: TabId, state: ShieldView },
    /// Etat des listes de filtres et date du dernier rafraichissement.
    FilterListsChanged { lists: Vec<FilterListView>, refreshed_at: Option<i64> },
    ExtensionsChanged { extensions: Vec<ExtensionView>, restart_pending: bool },
    /// Quelle fenetre d'extension est ouverte, pour que son icone se marque.
    ExtensionPopupChanged { id: Option<String> },
    BookmarksChanged { bookmarks: Vec<BookmarkView> },
    HistoryChanged { entries: Vec<HistoryEntryView>, total: usize },
    DownloadsChanged { downloads: Vec<DownloadView> },
    SettingsChanged { settings: Vec<SettingView> },
    /// La page est passee en plein ecran, ou en est sortie : l'interface s'efface.
    FullscreenChanged { active: bool },
    /// Le coeur demande le focus sur le champ d'adresse (raccourci clavier).
    FocusAddressRequested,
    /// Le navigateur va se relancer : l'interface montre son ecran d'attente.
    Restarting { reason: String },
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
    /// Avancement du chargement, de 0 a 1.
    pub progress: f32,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    /// Adresse de l'icone du site, servie par le coeur.
    pub favicon: Option<String>,
    pub security: Security,
    pub pinned: bool,
    pub zoom: f32,
    /// Vrai si la page joue du son.
    pub audible: bool,
    /// Vrai si l'onglet a ete mis en sommeil pour economiser la memoire.
    pub asleep: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Security {
    Secure,
    Mixed,
    Invalid,
    Insecure,
    Local,
}

/// L'etat du bouclier pour l'onglet courant.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldView {
    pub enabled: bool,
    pub active_here: bool,
    pub blocked_here: u64,
    pub blocked_total: u64,
}

/// Une liste de filtres souscrite par le bouclier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterListView {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    /// Nombre de regles chargees, quand la liste est active.
    pub rules: Option<usize>,
}

/// Une extension telle que l'interface l'affiche.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionView {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    /// Vrai tant que l'etat affiche ne correspond pas a ce qui tourne reellement.
    pub pending: bool,
    /// Vrai si l'interface peut la retirer elle-meme.
    pub removable: bool,
    /// Adresse de son icone, quand le paquet en fournit une.
    pub icon: Option<String>,
    /// Adresse de sa fenetre, quand elle en declare une. Sans elle, l'extension n'a
    /// rien a montrer : son icone declenche son action et c'est tout.
    pub popup: Option<String>,
    /// Adresse de sa page de reglages, quand elle en propose une.
    pub options: Option<String>,
}

/// Un rectangle de l'interface, en pixels, repere depuis le coin haut-gauche de la
/// fenetre. Sert d'ancre a ce qui s'affiche au-dessus de la page.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    /// `rename_all` ne renomme que les variantes : sans `rename_all_fields`, un champ en
    /// deux mots reste en `snake_case` cote Rust alors que le miroir TypeScript ecrit du
    /// `camelCase`. La demande partait, le coeur la rejetait, et le bouton recharger ne
    /// faisait rien. Ce test lit les formes exactes que l'interface envoie.
    #[test]
    fn les_champs_en_deux_mots_suivent_le_miroir_typescript() {
        let reload: UiRequest =
            serde_json::from_str(r#"{"kind":"reload","id":1,"bypassCache":true}"#).expect("reload");
        assert!(matches!(reload, UiRequest::Reload { id: 1, bypass_cache: true }));

        let oubli: UiRequest =
            serde_json::from_str(r#"{"kind":"removeHistoryEntry","url":"https://x/","visitedAt":42}"#)
                .expect("removeHistoryEntry");
        assert!(matches!(oubli, UiRequest::RemoveHistoryEntry { visited_at: 42, .. }));
    }

    /// Meme regle dans l'autre sens : ce que le coeur emet doit se lire cote interface.
    #[test]
    fn les_evenements_sortent_en_camel_case() {
        let evenement = CoreEvent::ExtensionsChanged { extensions: Vec::new(), restart_pending: true };
        let json = serde_json::to_string(&evenement).expect("serialisation");
        assert!(json.contains("\"restartPending\":true"), "{json}");
        assert!(!json.contains("restart_pending"), "{json}");
    }
}
