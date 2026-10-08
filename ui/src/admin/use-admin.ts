// Responsabilite : etat de la section Administration — relit le tableau de bord a l'ouverture, toutes les minutes et a
// chaque recherche (attente de 250 ms), et porte les actions sur les comptes.

import { useEffect, useMemo, useState } from 'react'
import type { UiRequest } from '../shared/contract'
import type { CoreState } from '../shared/core-state'
import {
  readAccounts, readDetail, readSummary, type AdminAccount, type AdminDetail, type AdminSummary,
} from './admin-data'

const REFRESH_MS = 60_000
const TYPING_MS = 250

export interface AdminController {
  summary: AdminSummary | null
  accounts: AdminAccount[]
  error: string | null
  query: string
  search: (next: string) => void
  refresh: () => void
  signOut: (id: string) => void
  remove: (id: string) => void
  setAdmin: (id: string, admin: boolean) => void
  /** Compte dont la fiche est ouverte, et sa fiche une fois recue. */
  opened: string | null
  detail: AdminDetail | null
  open: (id: string | null) => void
}

export function useAdmin(
  send: (request: UiRequest) => void,
  data: CoreState['admin'],
  account: CoreState['adminAccount'],
): AdminController {
  const [query, setQuery] = useState('')
  const [opened, setOpened] = useState<string | null>(null)
  useEffect(() => {
    if (opened !== null) send({ kind: 'adminAccountDetail', id: opened })
  }, [send, opened, data])
  const detail = useMemo(() => readDetail(account?.detail), [account])
  useEffect(() => {
    const wait = setTimeout(() => send({ kind: 'adminRefresh', query }), TYPING_MS)
    const every = setInterval(() => send({ kind: 'adminRefresh', query }), REFRESH_MS)
    return () => {
      clearTimeout(wait)
      clearInterval(every)
    }
  }, [send, query])
  const summary = useMemo(() => readSummary(data?.summary), [data])
  const accounts = useMemo(() => readAccounts(data?.accounts), [data])
  return {
    summary,
    accounts,
    error: data?.error ?? null,
    query,
    search: setQuery,
    refresh: () => send({ kind: 'adminRefresh', query }),
    signOut: (id) => send({ kind: 'adminSignOutAccount', id }),
    remove: (id) => send({ kind: 'adminDeleteAccount', id }),
    setAdmin: (id, admin) => send({ kind: 'adminSetFlag', id, admin }),
    opened,
    detail: detail !== null && detail.account.id === opened ? detail : null,
    open: setOpened,
  }
}
