// Responsabilite : traduire les gestes de la barre en requetes du contrat, pour l'onglet actif.

import { useMemo } from 'react'
import type { TabId, UiRequest } from '../shared/contract'

export interface SidebarActions {
  newTab: (url?: string) => void
  selectTab: (id: TabId) => void
  closeTab: (id: TabId) => void
  navigate: (input: string) => void
  back: () => void
  forward: () => void
  reload: () => void
  stop: () => void
  toggleShieldSite: () => void
  setShieldEnabled: (enabled: boolean) => void
  refreshLists: () => void
  devTools: () => void
}

type Send = (request: UiRequest) => void

function buildActions(send: Send, activeId: TabId | null): SidebarActions {
  const onActive =
    (make: (id: TabId) => UiRequest) =>
    (): void => {
      if (activeId !== null) send(make(activeId))
    }
  return {
    newTab: (url) => send(url === undefined ? { kind: 'newTab' } : { kind: 'newTab', url }),
    selectTab: (id) => send({ kind: 'selectTab', id }),
    closeTab: (id) => send({ kind: 'closeTab', id }),
    navigate: (input) => onActive((id) => ({ kind: 'navigate', id, input }))(),
    back: onActive((id) => ({ kind: 'goBack', id })),
    forward: onActive((id) => ({ kind: 'goForward', id })),
    reload: onActive((id) => ({ kind: 'reload', id, bypassCache: false })),
    stop: onActive((id) => ({ kind: 'stop', id })),
    toggleShieldSite: onActive((id) => ({ kind: 'toggleShieldForSite', id })),
    setShieldEnabled: (enabled) => send({ kind: 'setShieldEnabled', enabled }),
    refreshLists: () => send({ kind: 'refreshFilterLists', force: true }),
    devTools: onActive((id) => ({ kind: 'openDevTools', id })),
  }
}

export function useSidebarActions(send: Send, activeId: TabId | null): SidebarActions {
  return useMemo(() => buildActions(send, activeId), [send, activeId])
}
