// Responsabilite : traduire les gestes sur les onglets en requetes du contrat.

import { useMemo } from 'react'
import type { TabId, TabView, UiRequest } from '../shared/contract'

export interface TabActions {
  newTab: (url?: string) => void
  select: (id: TabId) => void
  close: (id: TabId) => void
  pin: (id: TabId, pinned: boolean) => void
  /** Deplace un onglet a une position dans la liste complete (epingles compris). */
  move: (id: TabId, to: number) => void
  setZoom: (id: TabId, factor: number) => void
  addBookmark: (id: TabId) => void
  reload: (id: TabId) => void
  // --- Onglet actif ---
  navigate: (input: string) => void
  back: () => void
  forward: () => void
  reloadActive: () => void
  stop: () => void
  devTools: () => void
}

type Send = (request: UiRequest) => void

function buildActions(send: Send, activeId: TabId | null): TabActions {
  const onActive =
    (make: (id: TabId) => UiRequest) =>
    (): void => {
      if (activeId !== null) send(make(activeId))
    }
  return {
    newTab: (url) => send(url === undefined ? { kind: 'newTab' } : { kind: 'newTab', url }),
    select: (id) => send({ kind: 'selectTab', id }),
    close: (id) => send({ kind: 'closeTab', id }),
    pin: (id, pinned) => send({ kind: 'pinTab', id, pinned }),
    move: (id, to) => send({ kind: 'moveTab', id, to }),
    setZoom: (id, factor) => send({ kind: 'setZoom', id, factor }),
    addBookmark: (id) => send({ kind: 'addBookmark', id }),
    reload: (id) => send({ kind: 'reload', id, bypassCache: false }),
    navigate: (input) => onActive((id) => ({ kind: 'navigate', id, input }))(),
    back: onActive((id) => ({ kind: 'goBack', id })),
    forward: onActive((id) => ({ kind: 'goForward', id })),
    reloadActive: onActive((id) => ({ kind: 'reload', id, bypassCache: false })),
    stop: onActive((id) => ({ kind: 'stop', id })),
    devTools: onActive((id) => ({ kind: 'openDevTools', id })),
  }
}

export function useTabActions(send: Send, activeId: TabId | null): TabActions {
  return useMemo(() => buildActions(send, activeId), [send, activeId])
}

/** Position finale d'un onglet dans la liste complete, quand seule la partie libre a ete reordonnee. */
export function targetIndex(all: TabView[], reorderedLoose: TabView[], id: TabId): number {
  const pinned = all.filter((tab) => tab.pinned)
  const full = [...pinned, ...reorderedLoose]
  return full.findIndex((tab) => tab.id === id)
}
