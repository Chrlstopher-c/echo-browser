// Responsabilite : miroir TypeScript du contrat d'echange avec le coeur Rust.
// Jumeau de core/contract/src/lib.rs — les deux se modifient ensemble, jamais l'un sans l'autre.

export type TabId = number
export type DownloadId = number

/** Ce que l'interface demande au coeur. */
export type UiRequest =
  // --- Onglets et navigation ---
  | { kind: 'newTab'; url?: string }
  | { kind: 'closeTab'; id: TabId }
  | { kind: 'sleepTab'; id: TabId }
  | { kind: 'selectTab'; id: TabId }
  /** Deplace un onglet a une nouvelle position dans la liste. */
  | { kind: 'moveTab'; id: TabId; to: number }
  | { kind: 'pinTab'; id: TabId; pinned: boolean }
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
  /** Le navigateur va se relancer : l'interface montre son ecran d'attente. */
  | { kind: 'restarting'; reason: string }
  | { kind: 'notice'; level: NoticeLevel; message: string }
  /** Un site demande une permission : l'interface pose la question. */
  | { kind: 'permissionRequested'; id: number; origin: string; kinds: string[] }
  | { kind: 'permissionResolved'; id: number }

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
  zoom: number
  /** Vrai si la page joue du son. */
  audible: boolean
  /** Vrai si l'onglet a ete mis en sommeil pour economiser la memoire. */
  asleep: boolean
}

export type Security = 'secure' | 'mixed' | 'invalid' | 'insecure' | 'local'

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
