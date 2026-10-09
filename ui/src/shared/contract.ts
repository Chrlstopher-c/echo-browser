// Responsabilite : miroir TypeScript du contrat d'echange avec le coeur Rust.
// Jumeau de core/contract/src/lib.rs — les deux se modifient ensemble, jamais l'un sans l'autre.

import type { AccountView, RemoteMachineView, VaultKindView } from './contract-account'
import type { NetworkView, RoutineProposalView, RoutineView } from './contract-network'

export type { AccountView, RemoteMachineView, SyncMode, VaultKindView } from './contract-account'
export type {
  JournalEntryView, NetDomainView, NetRequestView, NetWeightView, NetworkView, RoutineProposalView, RoutineView,
} from './contract-network'
export type TabId = number
export type DownloadId = number

/** Ce que l'interface demande au coeur. */
export type UiRequest =
  // --- Onglets et navigation ---
  | { kind: 'newTab'; url?: string; container?: string }
  /** Rouvre un onglet dans un autre conteneur (cookies et comptes a part), ou dans le commun (null). */
  | { kind: 'setTabContainer'; id: TabId; container: string | null }
  | { kind: 'closeTab'; id: TabId }
  /** Reveille d'avance un onglet endormi que la souris survole. */
  | { kind: 'warmTab'; id: TabId }
  /** Ouvre une page pleine largeur d'Echo dans un onglet, ou y revient. */
  | { kind: 'openPage'; page: 'reglages' | 'bibliotheque' | 'extensions' | 'admin' | 'aide' | 'effacer' }
  /** Suggestions pour ce que l'utilisateur tape dans l'adresse. */
  | { kind: 'suggest'; query: string }
  | { kind: 'find'; text: string; forward: boolean; next: boolean }
  | { kind: 'stopFind' }
  | { kind: 'setPageTheme'; theme: Record<string, unknown> }
  | { kind: 'setSchemeChoice'; choice: 'light' | 'dark' | 'system' }
  | { kind: 'setTabMuted'; id: TabId; muted: boolean }
  | { kind: 'importSources' }
  | { kind: 'importBrowser'; id: string }
  | { kind: 'fillForm'; index: number }
  | { kind: 'clearBrowsingData'; since: number; history: boolean; cookies: boolean; cache: boolean }
  /** Ouvrir un panneau de la barre (`network`, `shield`, `extensions`) — depuis l'Aide par exemple. */
  | { kind: 'openSidebarSheet'; sheet: string }
  /** Referme les outils de developpement ancres. */
  | { kind: 'closeDevTools' }
  /** La poignee de la fenetre d'extension a ete tiree. */
  | { kind: 'resizeExtensionPopup'; dx: number; dy: number }
  /** La poignee des outils a ete tiree de `dx` pixels. */
  | { kind: 'resizeDevTools'; dx: number }
  | { kind: 'sleepTab'; id: TabId }
  | { kind: 'selectTab'; id: TabId }
  /** Deplace un onglet a une nouvelle position dans la liste. */
  | { kind: 'moveTab'; id: TabId; to: number }
  | { kind: 'pinTab'; id: TabId; pinned: boolean }
  /** Garde l'onglet toujours eveille : jamais endormi ni allege. */
  | { kind: 'keepTabAwake'; id: TabId; keep: boolean }
  /** Range un onglet dans un dossier, ou l'en sort (null). */
  /** Oublie une decision de permission retenue : la question sera reposee. */
  | { kind: 'forgetPermission'; origin: string; permission: string }
  | { kind: 'setTabFolder'; id: TabId; folder: string | null }
  | { kind: 'navigate'; id: TabId; input: string }
  | { kind: 'goBack'; id: TabId }
  | { kind: 'goForward'; id: TabId }
  | { kind: 'reload'; id: TabId; bypassCache: boolean }
  | { kind: 'stop'; id: TabId }
  /** Facteur de zoom de la page, 1.0 etant la taille naturelle. */
  | { kind: 'setZoom'; id: TabId; factor: number }
  | { kind: 'revealSidebar'; reveal: boolean }
  | { kind: 'openDevTools'; id: TabId }
  | { kind: 'openTerminal' }
  /** Reponse a une demande de permission d'un site. */
  | { kind: 'answerPermission'; id: number; allow: boolean; remember: boolean }
  /** Sort du plein ecran, quand l'utilisateur le demande depuis l'interface. */
  | { kind: 'exitFullscreen' }
  // --- Mise en page ---
  /** L'interface reclame une largeur : le coeur repositionne la vue du contenu a sa droite. */
  | { kind: 'setChromeWidth'; pixels: number }
  | { kind: 'setSidebarCollapsed'; collapsed: boolean }
  /** Teinte dominante de l'espace courant, appliquee au cadre autour de la page. */
  | { kind: 'setAccent'; color: string }
  /** Theme d'Echo, transmis aux pages comme `prefers-color-scheme`. */
  | { kind: 'setColorScheme'; dark: boolean }
  /** Change de profil : ses onglets s'affichent, avec ses propres comptes. */
  | { kind: 'setSpace'; id: string }
  // --- Bouclier ---
  | { kind: 'toggleShieldForSite'; id: TabId }
  | { kind: 'setShieldEnabled'; enabled: boolean }
  | { kind: 'refreshFilterLists'; force: boolean }
  | { kind: 'setFilterListEnabled'; id: string; enabled: boolean }
  // --- Extensions ---
  /** Ouvre la fiche d'une extension, ou le catalogue, pour que Chromium l'installe. */
  | { kind: 'installExtension'; source: string }
  /** Ouvre le catalogue dans un onglet, pour y chercher une extension. */
  | { kind: 'openCatalog' }
  | { kind: 'removeExtension'; id: string }
  | { kind: 'setExtensionEnabled'; id: string; enabled: boolean }
  | { kind: 'openExtensionManager' }
  /** Ouvre la fenetre d'une extension, ancree sous son icone. */
  | { kind: 'openExtensionPopup'; id: string; anchor: AnchorRect }
  /** Referme la fenetre d'extension ouverte, s'il y en a une. */
  | { kind: 'closeExtensionPopup' }
  /** Donne au coeur les couleurs de l'espace, pour ce qui s'affiche au-dessus de la page. */
  | { kind: 'setOverlayTheme'; theme: OverlayTheme }
  /** Declenche une action du menu contextuel. */
  | { kind: 'runContextMenu'; action: MenuItemKind }
  /** Referme le menu contextuel. */
  | { kind: 'closeContextMenu' }
  /** Ouvre la page de reglages d'une extension dans un onglet. */
  | { kind: 'openExtensionOptions'; id: string }
  // --- Bibliotheque ---
  | { kind: 'addBookmark'; id: TabId }
  | { kind: 'removeBookmark'; url: string }
  | { kind: 'moveBookmark'; url: string; to: number }
  | { kind: 'removeHistoryEntry'; url: string; visitedAt: number }
  | { kind: 'clearHistory' }
  /** Filtre l'historique. Une requete vide rend les entrees les plus recentes. */
  | { kind: 'searchHistory'; terms: string }
  // --- Telechargements ---
  | { kind: 'openDownload'; id: DownloadId }
  /** Ouvre le dossier contenant le fichier. */
  | { kind: 'revealDownload'; id: DownloadId }
  | { kind: 'cancelDownload'; id: DownloadId }
  /** Retire l'entree de la liste, sans effacer le fichier. */
  | { kind: 'forgetDownload'; id: DownloadId }
  // --- Reglages ---
  | { kind: 'updateSetting'; key: string; value: SettingValue }
  // --- Cycle de vie ---
  /** Relance le navigateur. Les onglets ouverts sont retrouves apres la relance. */
  | { kind: 'restartBrowser' }
  // --- Decodeurs video ---
  /** Telecharge le decodeur complet (H.264/AAC) depuis un tiers ; actif a la prochaine relance. */
  | { kind: 'installVideoCodecs' }
  | { kind: 'removeVideoCodecs' }
  /** Ferme la proposition d'installation ; `forever` : ne plus jamais la faire. */
  | { kind: 'dismissVideoCodecs'; forever: boolean }
  // --- Mise a jour ---
  /** Verifie la derniere release et la prepare si elle est plus recente. */
  | { kind: 'checkForUpdates' }
  // --- Compte Echo ---
  /** Se connecter, ou creer le compte (`create`). Le mot de passe ne quitte pas la machine. */
  | { kind: 'accountSignIn'; email: string; password: string; create: boolean }
  | { kind: 'accountSignOut' }
  | { kind: 'accountSync' }
  /** Lire ce que le service garde du compte (dechiffre ici), pour le montrer. */
  | { kind: 'accountInspect' }
  /** Supprimer le compte et toutes ses donnees du service, puis se deconnecter. Definitif. */
  | { kind: 'accountDelete' }
  // --- Reseau (panneau de l'onglet actif) ---
  /** Le panneau Reseau s'ouvre ou se ferme : le coeur ne diffuse que pendant qu'il est ouvert. */
  | { kind: 'networkWatch'; on: boolean }
  /** Detail d'un domaine (ou de toutes les requetes avec `null`). */
  | { kind: 'networkFocus'; host: string | null }
  /** Bloquer ou debloquer un hote sur le site de l'onglet actif. */
  | { kind: 'networkBlockHost'; host: string; blocked: boolean }
  /** Isolement strict du site de l'onglet actif : aucune requete vers un autre site. */
  | { kind: 'networkSetStrict'; strict: boolean }
  // --- Profils ---
  /** Supprimer (`delete`) ou reinitialiser un profil ; le profil principal ne se supprime pas. */
  | { kind: 'profileForget'; id: string; delete: boolean }
  // --- Routines (suites de sites ouvertes souvent) ---
  | { kind: 'routineAccept'; fingerprint: string; name: string }
  | { kind: 'routineDismiss'; fingerprint: string }
  | { kind: 'routineOpen'; id: number }
  | { kind: 'routineRemove'; id: number }
  // --- Administration (comptes administrateurs seulement ; le service verifie a chaque appel) ---
  /** Relire le tableau de bord, comptes filtres par `query` (adresse e-mail). */
  | { kind: 'adminRefresh'; query: string }
  | { kind: 'adminSignOutAccount'; id: string }
  /** Supprimer un compte et tout son coffre. Definitif. */
  | { kind: 'adminDeleteAccount'; id: string }
  | { kind: 'adminSetFlag'; id: string; admin: boolean }
  /** Fiche d'un compte : usage par jour et par action, machines, coffre par type. */
  | { kind: 'adminAccountDetail'; id: string }

/** Ce que le coeur renvoie a l'interface. */
export type CoreEvent =
  | { kind: 'tabsChanged'; tabs: TabView[]; active: TabId | null }
  | { kind: 'tabUpdated'; tab: TabView }
  | { kind: 'shieldUpdated'; id: TabId; state: ShieldView }
  /** Etat des listes de filtres et date du dernier rafraichissement. */
  | { kind: 'filterListsChanged'; lists: FilterListView[]; refreshedAt: number | null }
  | { kind: 'extensionsChanged'; extensions: ExtensionView[]; restartPending: boolean }
  /** Quelle fenetre d'extension est ouverte, pour que son icone se marque. */
  | { kind: 'extensionPopupChanged'; id: string | null }
  | { kind: 'bookmarksChanged'; bookmarks: BookmarkView[] }
  | { kind: 'historyChanged'; entries: HistoryEntryView[]; total: number }
  | { kind: 'downloadsChanged'; downloads: DownloadView[] }
  | { kind: 'settingsChanged'; settings: SettingView[] }
  /** La page est passee en plein ecran, ou en est sortie : l'interface s'efface. */
  | { kind: 'fullscreenChanged'; active: boolean }
  /** Le coeur demande le focus sur le champ d'adresse (raccourci clavier). */
  | { kind: 'focusAddressRequested' }
  | { kind: 'findRequested' }
  | { kind: 'windowNarrow'; narrow: boolean }
  | { kind: 'pageTheme'; theme: unknown }
  | { kind: 'schemeChoiceRequested'; choice: string }
  | { kind: 'formOffer'; actions: NoticeAction[] }
  | { kind: 'importSources'; sources: ImportSourceView[] }
  | { kind: 'toggleSidebarRequested' }
  | { kind: 'findResult'; count: number; current: number }
  /** Le navigateur va se relancer : l'interface montre son ecran d'attente. */
  | { kind: 'restarting'; reason: string }
  | { kind: 'notice'; level: NoticeLevel; message: string; actions: NoticeAction[] }
  /** Un site demande une permission : l'interface pose la question. */
  | { kind: 'permissionRequested'; id: number; origin: string; kinds: string[] }
  | { kind: 'permissionResolved'; id: number }
  /** Les decisions de permission retenues, par site. */
  | { kind: 'permissionsChanged'; grants: PermissionGrantView[] }
  | { kind: 'videoCodecsChanged'; codecs: VideoCodecsView }
  | { kind: 'updateChanged'; update: UpdateView }
  | { kind: 'accountChanged'; account: AccountView }
  /** Onglets ouverts sur les autres machines du compte. */
  | { kind: 'remoteTabsChanged'; machines: RemoteMachineView[] }
  /** Ce que le service garde du compte, en reponse a `accountInspect`. */
  | { kind: 'accountVault'; kinds: VaultKindView[] }
  /** Tableau de bord tel que le service le rend (forme lue par `admin/admin-data.ts`), ou l'erreur. */
  | { kind: 'adminData'; summary: unknown; accounts: unknown; signals: unknown; error: string | null }
  /** Suggestions de l'adresse, pour `query`. */
  | { kind: 'suggestions'; query: string; items: SuggestionView[] }
  /** La barre doit ouvrir ce panneau. */
  | { kind: 'sidebarSheetRequested'; sheet: string }
  /** Une page surveillee a change depuis la visite precedente. */
  | { kind: 'pageChanged'; url: string; added: string[]; removed: string[] }
  /** Une suite de sites revient : proposition de routine. */
  | { kind: 'routineProposed'; proposal: RoutineProposalView }
  | { kind: 'routinesChanged'; routines: RoutineView[] }
  /** Reseau de l'onglet actif (panneau ouvert seulement). */
  | { kind: 'networkChanged'; network: NetworkView }
  /** Fiche d'un compte telle que le service la rend (lue par `admin/admin-data.ts`), ou l'erreur. */
  | { kind: 'adminAccount'; detail: unknown; error: string | null }
  /** Theme du bureau ; null : aucune preference exprimee. */
  | { kind: 'systemScheme'; dark: boolean | null }

/** Ou en est la mise a jour de la version installee. */
export type UpdateStatus =
  | 'unavailable'
  | 'upToDate'
  | 'checking'
  | 'available'
  | 'downloading'
  | 'ready'
  | 'failed'

export interface UpdateView {
  current: string
  status: UpdateStatus
  /** Derniere version publiee, quand elle est connue. */
  latest: string | null
  error: string | null
}

/** Ou en est le decodeur video complet (H.264/AAC). */
export type VideoCodecsStatus =
  | 'builtIn'
  | 'missing'
  | 'downloading'
  | 'pendingRestart'
  | 'active'
  | 'pendingRemoval'
  | 'unavailable'

export interface VideoCodecsView {
  status: VideoCodecsStatus
  /** D'ou vient le decodeur telechargeable. */
  source: string
  error: string | null
  /** Site dont une video attend le decodeur : l'interface propose de l'installer. */
  proposal: string | null
}

/** L'etat d'un onglet tel que l'interface l'affiche. */
export interface TabView {
  id: TabId
  title: string
  url: string
  loading: boolean
  /** Avancement du chargement, de 0 a 1. */
  progress: number
  canGoBack: boolean
  canGoForward: boolean
  /** Adresse de l'icone du site, servie par le coeur. */
  favicon: string | null
  security: Security
  pinned: boolean
  /** Identifiant du dossier d'onglets, ou null. */
  folder: string | null
  /** Conteneur de l'onglet, ou null pour le contexte commun. */
  container: string | null
  /** Profil (espace) de l'onglet. */
  space: string
  zoom: number
  /** Vrai si la page joue du son. */
  audible: boolean
  /** Son coupe par l'utilisateur. */
  muted: boolean
  /** Vrai si l'onglet a ete mis en sommeil pour economiser la memoire. */
  asleep: boolean
  /** L'utilisateur l'a demande toujours eveille. */
  keepAwake: boolean
}

export type Security = 'secure' | 'mixed' | 'invalid' | 'insecure' | 'local' | 'failed'

/** L'etat du bouclier pour l'onglet courant. */
export interface ShieldView {
  enabled: boolean
  activeHere: boolean
  blockedHere: number
  blockedTotal: number
}

/** Une liste de filtres souscrite par le bouclier. */
export interface FilterListView {
  id: string
  title: string
  enabled: boolean
  /** Nombre de regles chargees, quand la liste est active. */
  rules: number | null
}

/** Une extension telle que l'interface l'affiche. */
/** Un rectangle de l'interface, repere depuis le coin haut-gauche de la fenetre. */
export interface OverlayTheme {
  shell: string
  card: string
  hover: string
  hairline: string
  ink: string
  inkMuted: string
  inkFaint: string
  /** Lumiere et ombre du relief neumorphique. */
  hi: string
  lo: string
  tint: string
  danger: string
}

export type MenuItemKind =
  | 'separator'
  | 'openLinkInTab'
  | 'openLinkInBackground'
  | 'copyLink'
  | 'saveLink'
  | 'openImage'
  | 'copyImageLink'
  | 'saveImage'
  | 'copyImage'
  | 'openMedia'
  | 'copyMediaLink'
  | 'saveMedia'
  | 'copy'
  | 'cut'
  | 'paste'
  | 'pastePlain'
  | 'selectAll'
  | 'searchSelection'
  | 'openSelection'
  | 'back'
  | 'forward'
  | 'reload'
  | 'copyPageLink'
  | 'bookmark'
  | 'savePage'
  | 'print'
  | 'toggleShield'
  | 'viewSource'
  | 'inspect'
  /** Masquer l'element clique, sur toutes les pages du meme gabarit. */
  | 'hideElement'
  /** Reafficher ce qui a ete masque sur le gabarit de cette page. */
  | 'unhideElements'
  /** Surveiller la page : signaler ce qui a change a la prochaine visite. */
  | 'watchPage'
  | 'fillForm1'
  | 'fillForm2'
  | 'fillForm3'
  | 'manageForms'
  | 'reader'
  | 'findInPage'
  | 'translatePage'
  | 'openLinkPrivate'
  | 'openLinkInContainer1'
  | 'openLinkInContainer2'
  | 'openLinkInContainer3'
  | 'unwatchPage'

export interface MenuEntry {
  kind: MenuItemKind
  label: string
  enabled: boolean
  separator: boolean
}

export interface ContextTarget {
  entries: MenuEntry[]
  link: string
  selection: string
}

export interface AnchorRect {
  x: number
  y: number
  width: number
  height: number
}

export interface ExtensionView {
  id: string
  name: string
  version: string
  enabled: boolean
  /** Vrai tant que l'etat affiche ne correspond pas a ce qui tourne reellement. */
  pending: boolean
  /** Vrai si l'interface peut la retirer elle-meme. */
  removable: boolean
  /** Adresse de son icone, quand le paquet en fournit une. */
  icon: string | null
  /** Adresse de sa fenetre, quand elle en declare une. */
  popup: string | null
  /** Adresse de sa page de reglages, quand elle en propose une. */
  options: string | null
  /** Ce que l'extension dit d'elle-meme. */
  description: string
  /** Les permissions que son manifeste reclame. */
  permissions: string[]
}

export interface BookmarkView {
  url: string
  title: string
  favicon: string | null
  addedAt: number
}

export interface HistoryEntryView {
  url: string
  title: string
  favicon: string | null
  visitedAt: number
  /** Nombre de visites sur cette adresse. */
  visits: number
}

export interface DownloadView {
  id: DownloadId
  fileName: string
  url: string
  /** Chemin complet, une fois le fichier ecrit. */
  path: string | null
  received: number
  total: number | null
  state: DownloadState
  startedAt: number
}

export type DownloadState = 'running' | 'paused' | 'complete' | 'cancelled' | 'failed'

/** Un reglage et sa valeur courante. */
export interface SettingView {
  key: string
  value: SettingValue
}

export type SettingValue =
  | { type: 'flag'; value: boolean }
  | { type: 'text'; value: string }
  | { type: 'number'; value: number }

export type NoticeLevel = 'info' | 'warning' | 'error'

/** Pont expose par le coeur dans la page d'interface. */
export interface CoreBridge {
  send(request: UiRequest): void
  subscribe(listener: (event: CoreEvent) => void): () => void
}

declare global {
  interface Window {
    echo?: CoreBridge
  }
}

/** Une decision de permission retenue pour un site. */
export interface PermissionGrantView {
  origin: string
  kind: string
  allow: boolean
}

/** Une suggestion de l'adresse : onglet ouvert (a activer), favori ou historique. */
export interface SuggestionView {
  /** « search » : les termes tapes ; « query » : une suggestion du moteur. */
  kind: 'search' | 'query' | 'tab' | 'bookmark' | 'history'
  title: string
  url: string
  tab: TabId | null
}

/** Bouton d'une notification : il renvoie sa requete au coeur. */
export interface NoticeAction {
  label: string
  request: UiRequest
}

/** Un navigateur installe dont Echo peut reprendre favoris et historique. */
export interface ImportSourceView {
  id: string
  name: string
}
