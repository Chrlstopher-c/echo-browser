// Responsabilite : commandes de l'historique — recherche differee, regroupement par jour, retrait,
// effacement confirme.

import { useCallback, useEffect, useMemo, useState } from 'react'
import type { HistoryEntryView, UiRequest } from '../../shared/contract'
import { startOfDay } from '../../shared/format'

/** Delai entre la derniere frappe et la requete au coeur. */
const SEARCH_DELAY_MS = 180

export interface HistoryDay {
  day: number
  entries: HistoryEntryView[]
}

export interface ClearConfirm {
  confirmingClear: boolean
  askClear: () => void
  cancelClear: () => void
  confirmClear: () => void
}

export interface HistoryController extends ClearConfirm {
  terms: string
  setTerms: (next: string) => void
  days: HistoryDay[]
  total: number
  shown: number
  remove: (entry: HistoryEntryView) => void
  open: (url: string) => void
}

interface Source {
  entries: HistoryEntryView[]
  total: number
}

type Send = (request: UiRequest) => void

export function groupByDay(entries: HistoryEntryView[]): HistoryDay[] {
  const days: HistoryDay[] = []
  for (const entry of entries) {
    const day = startOfDay(entry.visitedAt)
    const last = days.at(-1)
    if (last !== undefined && last.day === day) last.entries.push(entry)
    else days.push({ day, entries: [entry] })
  }
  return days
}

function useClearConfirm(send: Send): ClearConfirm {
  const [confirmingClear, setConfirming] = useState(false)
  const askClear = useCallback((): void => setConfirming(true), [])
  const cancelClear = useCallback((): void => setConfirming(false), [])
  const confirmClear = useCallback((): void => {
    setConfirming(false)
    send({ kind: 'clearHistory' })
  }, [send])
  return { confirmingClear, askClear, cancelClear, confirmClear }
}

export function useHistory(send: Send, source: Source): HistoryController {
  const { entries, total } = source
  const [terms, setTerms] = useState('')
  const clear = useClearConfirm(send)

  useEffect(() => {
    const timer = setTimeout(() => send({ kind: 'searchHistory', terms }), SEARCH_DELAY_MS)
    return () => clearTimeout(timer)
  }, [send, terms])

  const days = useMemo(() => groupByDay(entries), [entries])
  const remove = useCallback(
    (entry: HistoryEntryView): void =>
      send({ kind: 'removeHistoryEntry', url: entry.url, visitedAt: entry.visitedAt }),
    [send],
  )
  const open = useCallback((url: string): void => send({ kind: 'newTab', url }), [send])

  return { ...clear, terms, setTerms, days, total, shown: entries.length, remove, open }
}
