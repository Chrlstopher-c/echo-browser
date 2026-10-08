//! Contrat d'echange entre le coeur du navigateur et son interface.
//!
//! Ce fichier et `ui/src/shared/contract.ts` sont deux miroirs du meme contrat.
//! Modifier l'un sans l'autre casse l'interface en silence.

mod account;
mod network;

use serde::{Deserialize, Serialize};

pub use network::{JournalEntryView, NetDomainView, NetRequestView, NetworkView, RoutineProposalView, RoutineView};
pub use account::{AccountView, RemoteMachineView, RemoteTabView, VaultKindView, VaultLineView};

pub type TabId = u32;
pub type DownloadId = u32;

/// Ce que l'interface demande au coeur.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UiRequest {
    // --- Onglets et navigation ---
    NewTab { url: Option<String>, container: Option<String> },
    /// Rouvre un onglet dans un autre conteneur (ses cookies et comptes changent), ou dans le commun (`None`).
    SetTabContainer { id: TabId, container: Option<String> },
    CloseTab { id: TabId },
    SelectTab { id: TabId },
    /// Reveille d'avance un onglet endormi que la souris survole : son clic sera instantane.
    WarmTab { id: TabId },
    /// Endort un onglet inactif : sa page est dechargee, elle se recharge a la selection.
    SleepTab { id: TabId },
    /// Deplace un onglet a une nouvelle position dans la liste.
    MoveTab { id: TabId, to: usize },
    PinTab { id: TabId, pinned: bool },
    /// Garde l'onglet toujours eveille : jamais endormi ni allege.
    KeepTabAwake { id: TabId, keep: bool },
    /// Range un onglet dans un dossier, ou l'en sort (`None`).
    SetTabFolder { id: TabId, folder: Option<String> },
    Navigate { id: TabId, input: String },
    GoBack { id: TabId },
    GoForward { id: TabId },
    Reload { id: TabId, bypass_cache: bool },
    Stop { id: TabId },
    /// Facteur de zoom de la page, 1.0 etant la taille naturelle.
    SetZoom { id: TabId, factor: f32 },
    OpenDevTools { id: TabId },
    /// Referme les outils de developpement ancres (croix de leur barre).
    CloseDevTools,
    /// La poignee de la fenetre d'extension a ete tiree.
    ResizeExtensionPopup { dx: i32, dy: i32 },
    /// La poignee entre la page et les outils a ete tiree de `dx` pixels.
    ResizeDevTools { dx: i32 },
    /// Ouvre le terminal de Claude Code, ou y revient s'il est deja ouvert.
    OpenTerminal,
    /// Ouvre une page pleine largeur d'Echo (« reglages », « bibliotheque ») dans un onglet, ou y revient.
    OpenPage { page: String },
    /// Oublie une decision de permission retenue : la question sera reposee.
    ForgetPermission { origin: String, permission: String },
    /// Reponse a une demande de permission d'un site (camera, micro, position…).
    AnswerPermission { id: u64, allow: bool, remember: bool },
    /// Sort du plein ecran, quand l'utilisateur le demande depuis l'interface.
    ExitFullscreen,

    // --- Mise en page ---
    /// L'interface reclame une largeur : le coeur repositionne la vue du contenu a sa droite.
    SetChromeWidth { pixels: u32 },
    SetSidebarCollapsed { collapsed: bool },
    /// La souris longe le bord gauche (ou quitte la barre) : montre ou cache la barre repliee.
    RevealSidebar { reveal: bool },
    /// Teinte dominante de l'espace courant, appliquee au cadre autour de la page.
    SetAccent { color: String },
    /// Theme clair ou sombre d'Echo : les pages le recoivent comme `prefers-color-scheme`.
    SetColorScheme { dark: bool },
    /// Change de profil : ses onglets s'affichent, les nouveaux y naissent, avec ses propres comptes.
    SetSpace { id: String },

    // --- Bouclier ---
    ToggleShieldForSite { id: TabId },
    SetShieldEnabled { enabled: bool },
    RefreshFilterLists { force: bool },
    /// Active ou desactive une liste de filtres.
    SetFilterListEnabled { id: String, enabled: bool },

    // --- Extensions ---
    /// Ouvre la fiche d'une extension, ou le catalogue, pour que Chromium l'installe.
    /// Installe une extension depuis le catalogue : identifiant ou adresse de sa fiche.
    InstallExtension { source: String },
    /// Ouvre le catalogue dans un onglet, pour y chercher une extension.
    OpenCatalog,
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

    // --- Menu contextuel ---
    /// Declenche une action du menu contextuel sur la derniere cible cliquee.
    /// Le champ ne peut pas s'appeler « kind » : c'est deja l'etiquette de l'enveloppe.
    RunContextMenu { action: MenuItemKind },
    /// Referme le menu contextuel sans rien declencher.
    CloseContextMenu,

    // --- Apparence des surimpressions ---
    /// Donne au coeur les couleurs de l'espace courant, pour que ce qui s'affiche
    /// au-dessus de la page — menu, fenetres — porte la meme matiere que la barre.
    SetOverlayTheme { theme: OverlayTheme },

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

    // --- Decodeurs video ---
    /// Telecharge le decodeur complet (H.264/AAC) depuis un tiers ; actif a la prochaine relance.
    InstallVideoCodecs,
    RemoveVideoCodecs,
    /// Ferme la proposition d'installation ; `forever` : ne plus jamais la faire.
    DismissVideoCodecs { forever: bool },

    // --- Mise a jour ---
    /// Verifie la derniere release et la prepare si elle est plus recente.
    CheckForUpdates,

    // --- Compte Echo ---
    /// Se connecter, ou creer le compte (`create`). Le mot de passe ne quitte pas la machine (cles derivees ici).
    AccountSignIn { email: String, password: String, create: bool },
    AccountSignOut,
    AccountSync,
    /// Lire ce que le service garde du compte (dechiffre ici), pour le montrer.
    AccountInspect,
    /// Supprimer le compte et toutes ses donnees du service, puis se deconnecter. Definitif.
    AccountDelete,

    // --- Reseau (panneau de l'onglet actif) ---
    /// Le panneau Reseau s'ouvre (`on`) ou se ferme : le coeur ne diffuse que pendant qu'il est ouvert.
    NetworkWatch { on: bool },
    /// Detail d'un domaine (ou de toutes les requetes avec `None`).
    NetworkFocus { host: Option<String> },
    /// Bloquer ou debloquer un hote sur le site de l'onglet actif.
    NetworkBlockHost { host: String, blocked: bool },
    /// Isolement strict du site de l'onglet actif : aucune requete vers un autre site.
    NetworkSetStrict { strict: bool },

    // --- Routines (suites de sites ouvertes souvent) ---
    RoutineAccept { fingerprint: String, name: String },
    RoutineDismiss { fingerprint: String },
    RoutineOpen { id: i64 },
    RoutineRemove { id: i64 },

    // --- Administration (comptes administrateurs seulement ; le service verifie a chaque appel) ---
    /// Relire le tableau de bord, comptes filtres par `query` (adresse e-mail).
    AdminRefresh { query: String },
    /// Deconnecter un compte de toutes ses machines.
    AdminSignOutAccount { id: String },
    /// Supprimer un compte et tout son coffre. Definitif.
    AdminDeleteAccount { id: String },
    /// Donner ou retirer l'acces administrateur.
    AdminSetFlag { id: String, admin: bool },
    /// Fiche d'un compte : usage par jour et par action, machines, coffre par type.
    AdminAccountDetail { id: String },
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
    /// Le clic droit demande un menu : voici ce qu'il propose, et ou le poser.
    ContextMenuRequested { target: ContextTarget, x: i32, y: i32 },
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
    /// Un site demande une permission : l'interface pose la question a l'utilisateur.
    PermissionRequested { id: u64, origin: String, kinds: Vec<String> },
    /// Les decisions de permission retenues, par site.
    PermissionsChanged { grants: Vec<PermissionGrantView> },
    /// La question n'a plus lieu d'etre (repondue, ou page partie).
    PermissionResolved { id: u64 },
    VideoCodecsChanged { codecs: VideoCodecsView },
    UpdateChanged { update: UpdateView },
    AccountChanged { account: AccountView },
    /// Onglets ouverts sur les autres machines du compte.
    RemoteTabsChanged { machines: Vec<RemoteMachineView> },
    /// Ce que le service garde du compte, en reponse a `AccountInspect`.
    AccountVault { kinds: Vec<VaultKindView> },
    /// Tableau de bord tel que le service le rend (`summary` : chiffres, `accounts` : comptes), ou l'erreur.
    AdminData { summary: serde_json::Value, accounts: serde_json::Value, error: Option<String> },
    /// Une suite de sites revient : proposition de routine.
    RoutineProposed { proposal: RoutineProposalView },
    RoutinesChanged { routines: Vec<RoutineView> },
    /// Reseau de l'onglet actif (panneau ouvert seulement).
    NetworkChanged { network: NetworkView },
    /// Fiche d'un compte telle que le service la rend, ou l'erreur.
    AdminAccount { detail: serde_json::Value, error: Option<String> },
}

/// Ou en est la mise a jour de la version installee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateStatus {
    /// Version de developpement : pas de mise a jour automatique.
    Unavailable,
    UpToDate,
    Checking,
    /// Plus recente, pas encore telechargee (mise a jour automatique coupee).
    Available,
    Downloading,
    /// Preparee : appliquee au prochain demarrage.
    Ready,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateView {
    pub current: String,
    pub status: UpdateStatus,
    /// Derniere version publiee, quand elle est connue.
    pub latest: Option<String>,
    pub error: Option<String>,
}

/// Ou en est le decodeur video complet (H.264/AAC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VideoCodecsStatus {
    /// Le moteur embarque deja le decodeur complet.
    BuiltIn,
    /// Decodeur libre seul : le complet peut etre telecharge.
    Missing,
    Downloading,
    /// Telecharge, actif a la prochaine relance.
    PendingRestart,
    /// Telecharge et charge.
    Active,
    /// Retire, le decodeur libre reprend a la prochaine relance.
    PendingRemoval,
    /// Decodeur integre au moteur sans emplacement separe : rien a installer.
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoCodecsView {
    pub status: VideoCodecsStatus,
    /// D'ou vient le decodeur telechargeable.
    pub source: String,
    pub error: Option<String>,
    /// Site dont une video attend le decodeur : l'interface propose de l'installer.
    pub proposal: Option<String>,
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
    /// Identifiant du dossier d'onglets, s'il y en a un.
    pub folder: Option<String>,
    /// Conteneur de l'onglet, s'il n'est pas dans le contexte commun.
    pub container: Option<String>,
    /// Profil (espace) de l'onglet.
    pub space: String,
    pub zoom: f32,
    /// Vrai si la page joue du son.
    pub audible: bool,
    /// Vrai si l'onglet a ete mis en sommeil pour economiser la memoire.
    pub asleep: bool,
    /// L'utilisateur l'a demande toujours eveille.
    pub keep_awake: bool,
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
    /// Ce que l'extension dit d'elle-meme.
    pub description: String,
    /// Les permissions que son manifeste reclame.
    pub permissions: Vec<String>,
}

/// Les quelques couleurs dont une surimpression a besoin pour se fondre dans l'espace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayTheme {
    pub shell: String,
    pub card: String,
    pub hover: String,
    pub hairline: String,
    pub ink: String,
    pub ink_muted: String,
    pub ink_faint: String,
    /// Lumiere et ombre du relief neumorphique, et teinte de l'espace.
    pub hi: String,
    pub lo: String,
    pub tint: String,
    pub danger: String,
}

impl Default for OverlayTheme {
    fn default() -> Self {
        Self {
            shell: "#222326".to_string(),
            card: "#222326".to_string(),
            hover: "#27282c".to_string(),
            hairline: "#34363b".to_string(),
            ink: "#ebeced".to_string(),
            ink_muted: "#a3a6ae".to_string(),
            ink_faint: "#6c707a".to_string(),
            hi: "rgba(255, 255, 255, 0.075)".to_string(),
            lo: "rgba(0, 0, 0, 0.7)".to_string(),
            tint: "#8f96a3".to_string(),
            danger: "#e5484d".to_string(),
        }
    }
}

/// Ce que propose le clic droit, construit par le coeur selon la cible.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextTarget {
    pub entries: Vec<MenuEntry>,
    /// Adresse du lien clique, vide s'il n'y en avait pas.
    pub link: String,
    /// Texte selectionne, ecourte pour l'affichage.
    pub selection: String,
}

/// Une entree du menu contextuel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuEntry {
    pub kind: MenuItemKind,
    pub label: String,
    pub enabled: bool,
    /// Vrai pour un simple trait de separation : ni libelle, ni action.
    pub separator: bool,
}

impl MenuEntry {
    pub fn new(kind: MenuItemKind, label: &str) -> Self {
        Self { kind, label: label.to_string(), enabled: true, separator: false }
    }

    pub fn separator() -> Self {
        Self { kind: MenuItemKind::Separator, label: String::new(), enabled: false, separator: true }
    }

    /// Grise l'entree quand la condition est vraie. L'entree reste affichee : une action
    /// qui disparait deplace les autres et se cherche du regard.
    pub fn disabled_when(mut self, condition: bool) -> Self {
        self.enabled = !condition;
        self
    }
}

/// Les actions que le menu contextuel sait declencher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MenuItemKind {
    Separator,
    OpenLinkInTab,
    OpenLinkInBackground,
    CopyLink,
    SaveLink,
    OpenImage,
    CopyImageLink,
    SaveImage,
    /// Copie l'image elle-meme (pas son adresse) dans le presse-papiers.
    CopyImage,
    OpenMedia,
    CopyMediaLink,
    SaveMedia,
    Copy,
    Cut,
    Paste,
    PastePlain,
    SelectAll,
    SearchSelection,
    OpenSelection,
    Back,
    Forward,
    Reload,
    CopyPageLink,
    Bookmark,
    SavePage,
    Print,
    ToggleShield,
    ViewSource,
    Inspect,
    /// Masquer l'element clique, sur toutes les pages du meme gabarit.
    HideElement,
    /// Reafficher ce qui a ete masque sur le gabarit de cette page.
    UnhideElements,
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

/// Une decision de permission retenue pour un site.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionGrantView {
    pub origin: String,
    pub kind: String,
    pub allow: bool,
}
