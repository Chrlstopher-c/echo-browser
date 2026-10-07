// Responsabilite : etat de l'interface derive des evenements du coeur. Reducteur pur, sans effet de bord.

import type {
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
  UpdateView,
  VideoCodecsView,
} from './contract'

export interface Notice {
  level: NoticeLevel
  message: string
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
    case 'focusAddressRequested':
      return { ...state, addressFocusToken: state.addressFocusToken + 1 }
    case 'restarting':
      return { ...state, restarting: event.reason }
    case 'notice':
      return { ...state, notice: { level: event.level, message: event.message, at: Date.now() } }
    case 'permissionRequested':
    case 'permissionsChanged':
    case 'permissionResolved':
      return reducePermissions(state, event)
    case 'videoCodecsChanged':
      return { ...state, codecs: event.codecs }
    case 'updateChanged':
      return { ...state, update: event.update }
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
