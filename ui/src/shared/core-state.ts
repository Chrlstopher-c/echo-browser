// Responsabilite : etat de l'interface derive des evenements du coeur. Reducteur pur, sans effet de bord.

import type { CoreEvent, NoticeLevel, ShieldView, TabId, TabView } from './contract'

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
  notice: Notice | null
  filterListCount: number | null
}

export const EMPTY_CORE_STATE: CoreState = {
  tabs: [],
  activeId: null,
  shields: new Map(),
  notice: null,
  filterListCount: null,
}

function withShield(state: CoreState, id: TabId, view: ShieldView): CoreState {
  const shields = new Map(state.shields)
  shields.set(id, view)
  return { ...state, shields }
}

export function reduceCore(state: CoreState, event: CoreEvent): CoreState {
  switch (event.kind) {
    case 'tabsChanged':
      return { ...state, tabs: event.tabs, activeId: event.active }
    case 'tabUpdated':
      return { ...state, tabs: state.tabs.map((tab) => (tab.id === event.tab.id ? event.tab : tab)) }
    case 'shieldUpdated':
      return withShield(state, event.id, event.state)
    case 'filterListsRefreshed':
      return { ...state, filterListCount: event.count }
    case 'notice':
      return { ...state, notice: { level: event.level, message: event.message, at: Date.now() } }
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
