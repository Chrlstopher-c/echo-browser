// Responsabilite : etat de l'interface derive des evenements du coeur. Reducteur pur, sans effet de bord.

import type { ImportSourceView, NoticeAction,
  BookmarkView,
  CoreEvent,
  DownloadView,
  ExtensionView,
  FilterListView,
  HistoryEntryView,
  NoticeLevel,
  PermissionGrantView,
  SettingView,
  ShieldView,
  TabId,
  TabView,
  AccountView,
  RemoteMachineView,
  VaultKindView,
  NetworkView,
  RoutineProposalView,
  RoutineView,
  SuggestionView,
  UpdateView,
  VideoCodecsView,
} from './contract'

export interface Notice {
  level: NoticeLevel
  message: string
  actions: NoticeAction[]
  /** Horodatage : sert de cle de rendu pour rejouer l'animation sur un message identique. */
  at: number
}

export interface CoreState {
  tabs: TabView[]
  activeId: TabId | null
  shields: Map<TabId, ShieldView>
  filterLists: FilterListView[]
  /** Dernier rafraichissement des listes, ou null tant que le coeur n'en a pas fait. */
  filterListsRefreshedAt: number | null
  extensions: ExtensionView[]
  /** Vrai quand au moins un changement d'extension attend la relance du navigateur. */
  restartPending: boolean
  /** Extension dont la fenetre est ouverte au-dessus de la page, ou null. */
  extensionPopupId: string | null
  bookmarks: BookmarkView[]
  history: HistoryEntryView[]
  /** Nombre total d'entrees d'historique, au-dela de celles livrees. */
  historyTotal: number
  downloads: DownloadView[]
  settings: SettingView[]
  fullscreen: boolean
  /** Incremente a chaque demande de focus du champ d'adresse venue du coeur. */
  addressFocusToken: number
  /** Raison de la relance en cours, ou null tant que le navigateur tourne normalement. */
  restarting: string | null
  notice: Notice | null
  /** Demandes de permission en attente de reponse, la plus ancienne d'abord. */
  permissions: PermissionRequest[]
  /** Decisions de permission retenues par site. */
  grants: PermissionGrantView[]
  /** Decodeur video complet (H.264/AAC), null tant que le coeur n'a rien dit. */
  codecs: VideoCodecsView | null
  /** Mise a jour de la version installee, null tant que le coeur n'a rien dit. */
  update: UpdateView | null
  /** Compte Echo, null tant que le coeur n'a rien dit. */
  account: AccountView | null
  /** Onglets des autres machines du compte. */
  remoteMachines: RemoteMachineView[]
  /** Contenu du coffre du compte, null tant qu'il n'a pas ete demande. */
  vault: VaultKindView[] | null
  /** Tableau de bord des createurs, null tant qu'il n'a pas ete demande. */
  admin: { summary: unknown; accounts: unknown; signals: unknown; error: string | null } | null
  /** Dernieres suggestions de l'adresse. */
  suggestions: { query: string; items: SuggestionView[] } | null
  /** Panneau de la barre demande (par l'Aide) : nom et jeton (chaque demande l'incremente). */
  sheetRequest: { sheet: string; token: number } | null
  /** Derniere page surveillee qui a change, en attente d'etre vue. */
  pageChange: { url: string; added: string[]; removed: string[] } | null
  /** Routine proposee, en attente d'une reponse. */
  routineProposal: RoutineProposalView | null
  routines: RoutineView[]
  /** Reseau de l'onglet actif, tant que le panneau est ouvert. */
  network: NetworkView | null
  /** Fiche du compte ouvert dans l'administration. */
  adminAccount: { detail: unknown; error: string | null } | null
  /** Theme du bureau, pour le choix « Système ». */
  systemDark: boolean | null
  /** Incremente a chaque Ctrl+F. */
  findToken: number
  /** Fenetre etroite : la barre se replie d'elle-meme. */
  narrow: boolean
  /** Theme relaye par le coeur aux pages d'Echo. */
  pageTheme: unknown
  /** Dernier choix de theme demande par une page, et son instant (pour rejouer un meme choix). */
  schemeRequest: { choice: string; at: number } | null
  /** Fiches proposees pour le champ ou l'utilisateur vient d'entrer. */
  formOffer: { actions: NoticeAction[]; at: number } | null
  /** Autres navigateurs trouves (null : pas encore demande). */
  importSources: ImportSourceView[] | null
  /** Incremente a chaque Ctrl+Alt+S. */
  sidebarToggleToken: number
  find: { count: number; current: number }
}

export interface PermissionRequest {
  id: number
  origin: string
  kinds: string[]
}

export const EMPTY_CORE_STATE: CoreState = {
  tabs: [],
  activeId: null,
  shields: new Map(),
  filterLists: [],
  filterListsRefreshedAt: null,
  extensions: [],
  restartPending: false,
  extensionPopupId: null,
  bookmarks: [],
  history: [],
  historyTotal: 0,
  downloads: [],
  settings: [],
  fullscreen: false,
  addressFocusToken: 0,
  restarting: null,
  notice: null,
  permissions: [],
  grants: [],
  codecs: null,
  update: null,
  account: null,
  remoteMachines: [],
  vault: null,
  admin: null,
  adminAccount: null,
  systemDark: null,
  findToken: 0,
  narrow: false,
  pageTheme: null,
  schemeRequest: null,
  formOffer: null,
  importSources: null,
  sidebarToggleToken: 0,
  find: { count: 0, current: 0 },
  network: null,
  routineProposal: null,
  pageChange: null,
  sheetRequest: null,
  suggestions: null,
  routines: [],
}

function withShield(state: CoreState, id: TabId, view: ShieldView): CoreState {
  const shields = new Map(state.shields)
  shields.set(id, view)
  return { ...state, shields }
}

type PermissionEvent = Extract<CoreEvent, { kind: 'permissionRequested' | 'permissionsChanged' | 'permissionResolved' }>

function reducePermissions(state: CoreState, event: PermissionEvent): CoreState {
  switch (event.kind) {
    case 'permissionRequested': {
      const request = { id: event.id, origin: event.origin, kinds: event.kinds }
      return { ...state, permissions: [...state.permissions, request] }
    }
    case 'permissionsChanged':
      return { ...state, grants: event.grants }
    case 'permissionResolved':
      return { ...state, permissions: state.permissions.filter((request) => request.id !== event.id) }
  }
}

type LibraryEvent = Extract<
  CoreEvent,
  { kind: 'bookmarksChanged' | 'historyChanged' | 'downloadsChanged' | 'settingsChanged' }
>

function reduceLibrary(state: CoreState, event: LibraryEvent): CoreState {
  switch (event.kind) {
    case 'bookmarksChanged':
      return { ...state, bookmarks: event.bookmarks }
    case 'historyChanged':
      return { ...state, history: event.entries, historyTotal: event.total }
    case 'downloadsChanged':
      return { ...state, downloads: event.downloads }
    case 'settingsChanged':
      return { ...state, settings: event.settings }
  }
}

export function reduceCore(state: CoreState, event: CoreEvent): CoreState {
  switch (event.kind) {
    case 'tabsChanged':
      return { ...state, tabs: event.tabs, activeId: event.active }
    case 'tabUpdated':
      return { ...state, tabs: state.tabs.map((t) => (t.id === event.tab.id ? event.tab : t)) }
    case 'shieldUpdated':
      return withShield(state, event.id, event.state)
    case 'filterListsChanged':
      return { ...state, filterLists: event.lists, filterListsRefreshedAt: event.refreshedAt }
    case 'extensionsChanged':
      return { ...state, extensions: event.extensions, restartPending: event.restartPending }
    case 'extensionPopupChanged':
      return { ...state, extensionPopupId: event.id }
    case 'bookmarksChanged':
    case 'historyChanged':
    case 'downloadsChanged':
    case 'settingsChanged':
      return reduceLibrary(state, event)
    case 'fullscreenChanged':
      return { ...state, fullscreen: event.active }
    case 'importSources':
      return { ...state, importSources: event.sources }
    case 'formOffer':
      return { ...state, formOffer: { actions: event.actions, at: Date.now() } }
    case 'schemeChoiceRequested':
      return { ...state, schemeRequest: { choice: event.choice, at: Date.now() } }
    case 'pageTheme':
      return { ...state, pageTheme: event.theme }
    case 'windowNarrow':
      return { ...state, narrow: event.narrow }
    case 'toggleSidebarRequested':
      return { ...state, sidebarToggleToken: state.sidebarToggleToken + 1 }
    case 'findRequested':
      return { ...state, findToken: state.findToken + 1 }
    case 'findResult':
      return { ...state, find: { count: event.count, current: event.current } }
    case 'focusAddressRequested':
      return { ...state, addressFocusToken: state.addressFocusToken + 1 }
    case 'restarting':
      return { ...state, restarting: event.reason }
    case 'notice':
      return { ...state, notice: { level: event.level, message: event.message, actions: event.actions, at: Date.now() } }
    case 'permissionRequested':
    case 'permissionsChanged':
    case 'permissionResolved':
      return reducePermissions(state, event)
    case 'videoCodecsChanged':
      return { ...state, codecs: event.codecs }
    case 'updateChanged':
      return { ...state, update: event.update }
    case 'accountChanged':
      return { ...state, account: event.account }
    case 'remoteTabsChanged':
      return { ...state, remoteMachines: event.machines }
    case 'accountVault':
      return { ...state, vault: event.kinds }
    case 'suggestions':
      return { ...state, suggestions: { query: event.query, items: event.items } }
    case 'sidebarSheetRequested':
      return { ...state, sheetRequest: { sheet: event.sheet, token: (state.sheetRequest?.token ?? 0) + 1 } }
    case 'pageChanged':
      return { ...state, pageChange: { url: event.url, added: event.added, removed: event.removed } }
    case 'routineProposed':
      return { ...state, routineProposal: event.proposal }
    case 'routinesChanged':
      return { ...state, routines: event.routines }
    case 'networkChanged':
      return { ...state, network: event.network }
    case 'adminAccount':
      return { ...state, adminAccount: { detail: event.detail, error: event.error } }
    case 'systemScheme':
      return { ...state, systemDark: event.dark }
    case 'adminData':
      return {
        ...state,
        admin: { summary: event.summary, accounts: event.accounts, signals: event.signals, error: event.error },
      }
  }
}

/** Onglet actif, ou null si aucun onglet. */
export function activeTabOf(state: CoreState): TabView | null {
  if (state.activeId === null) return null
  return state.tabs.find((tab) => tab.id === state.activeId) ?? null
}

/** Bouclier de l'onglet actif, avec un repli neutre tant que le coeur n'a rien envoye. */
export function activeShieldOf(state: CoreState): ShieldView {
  const fallback: ShieldView = { enabled: true, activeHere: true, blockedHere: 0, blockedTotal: 0 }
  if (state.activeId === null) return fallback
  return state.shields.get(state.activeId) ?? fallback
}
