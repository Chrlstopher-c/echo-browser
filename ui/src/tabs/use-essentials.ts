// Responsabilite : essentiels persistes localement, favicons rafraichis depuis les onglets ouverts.

import { useCallback, useEffect, useMemo, useState } from 'react'
import type { TabView } from '../shared/contract'
import { readLocal, writeLocal } from '../shared/local-store'
import { essentialFromTab, hostOf, isEssential, SEED_ESSENTIALS, splitTabs, type Essential } from './essentials-model'

const STORE_KEY = 'echo.essentials'

export interface EssentialsController {
  essentials: Essential[]
  /** Onglet ouvert derriere chaque essentiel, par hote. */
  represented: Map<string, TabView>
  /** Onglets sans essentiel, a lister. */
  loose: TabView[]
  pin: (tab: TabView) => void
  unpin: (host: string) => void
}

function readStored(simulated: boolean): Essential[] {
  const stored = readLocal(STORE_KEY, (raw) => (Array.isArray(raw) && raw.every(isEssential) ? raw : null))
  if (stored !== null) return stored
  return simulated ? SEED_ESSENTIALS : []
}

function refreshFavicons(essentials: Essential[], tabs: TabView[]): Essential[] {
  let changed = false
  const next = essentials.map((item) => {
    const tab = tabs.find((candidate) => candidate.favicon !== null && hostOf(candidate.url) === item.host)
    if (tab === undefined || tab.favicon === item.favicon) return item
    changed = true
    return { ...item, favicon: tab.favicon }
  })
  return changed ? next : essentials
}

export function useEssentials(tabs: TabView[], simulated: boolean): EssentialsController {
  const [essentials, setEssentials] = useState<Essential[]>(() => readStored(simulated))

  useEffect(() => {
    setEssentials((current) => refreshFavicons(current, tabs))
  }, [tabs])

  useEffect(() => {
    writeLocal(STORE_KEY, essentials)
  }, [essentials])

  const pin = useCallback((tab: TabView): void => {
    const item = essentialFromTab(tab)
    if (item.host.length === 0) return
    setEssentials((current) => (current.some((e) => e.host === item.host) ? current : [...current, item]))
  }, [])

  const unpin = useCallback((host: string): void => {
    setEssentials((current) => current.filter((item) => item.host !== host))
  }, [])

  const split = useMemo(() => splitTabs(tabs, essentials), [tabs, essentials])
  return { essentials, represented: split.represented, loose: split.loose, pin, unpin }
}
