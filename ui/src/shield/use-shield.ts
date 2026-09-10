// Responsabilite : commandes du bouclier — interrupteurs, listes de filtres, rafraichissement borne.

import { useCallback, useEffect, useState } from 'react'
import type { FilterListView, TabId, UiRequest } from '../shared/contract'

/** Au-dela, on cesse d'attendre le coeur : le bouton redevient cliquable. */
const REFRESH_TIMEOUT_MS = 20_000

export interface ShieldController {
  lists: FilterListView[]
  refreshedAt: number | null
  /** Vrai entre la demande de rafraichissement et la reponse du coeur. */
  refreshing: boolean
  setEnabled: (enabled: boolean) => void
  toggleSite: () => void
  setListEnabled: (id: string, enabled: boolean) => void
  refresh: () => void
}

interface Source {
  lists: FilterListView[]
  refreshedAt: number | null
  activeId: TabId | null
}

type Send = (request: UiRequest) => void

export function useShield(send: Send, source: Source): ShieldController {
  const { lists, refreshedAt, activeId } = source
  const [refreshing, setRefreshing] = useState(false)

  // Fin de l'attente : la date bouge, ou le garde-fou de temps parle.
  useEffect(() => {
    setRefreshing(false)
  }, [refreshedAt])

  useEffect(() => {
    if (!refreshing) return
    const timer = setTimeout(() => setRefreshing(false), REFRESH_TIMEOUT_MS)
    return () => clearTimeout(timer)
  }, [refreshing])

  const setEnabled = useCallback((enabled: boolean): void => send({ kind: 'setShieldEnabled', enabled }), [send])
  const toggleSite = useCallback((): void => {
    if (activeId !== null) send({ kind: 'toggleShieldForSite', id: activeId })
  }, [send, activeId])
  const setListEnabled = useCallback(
    (id: string, enabled: boolean): void => send({ kind: 'setFilterListEnabled', id, enabled }),
    [send],
  )
  const refresh = useCallback((): void => {
    setRefreshing(true)
    send({ kind: 'refreshFilterLists', force: true })
  }, [send])

  return { lists, refreshedAt, refreshing, setEnabled, toggleSite, setListEnabled, refresh }
}
