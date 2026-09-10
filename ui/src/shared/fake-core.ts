// Responsabilite : faux coeur en memoire pour le developpement de l'interface.
// Aucune logique metier reelle ici — il ne fait que rejouer des evenements plausibles.

import type { CoreBridge, CoreEvent, ShieldView, TabId, TabView, UiRequest } from './contract'
import { fallbackTitle, looksLikeUrl } from './url-shape'

const LOAD_STEPS = 14
const LOAD_TICK_MS = 90
const SEARCH_PREFIX = 'https://duckduckgo.com/?q='
const HOME_URL = 'https://anthropic.com/'

function toUrl(input: string): string {
  const value = input.trim()
  if (!looksLikeUrl(value)) return `${SEARCH_PREFIX}${encodeURIComponent(value)}`
  return /^[a-z][a-z0-9+.-]*:/i.test(value) ? value : `https://${value}`
}

function blankTab(id: TabId, url: string): TabView {
  return {
    id,
    title: fallbackTitle(url),
    url,
    loading: false,
    progress: 0,
    canGoBack: false,
    canGoForward: false,
    favicon: null,
  }
}

/** Faux coeur : etat en memoire, evenements simules, aucune persistance. */
class FakeCore implements CoreBridge {
  private tabs: TabView[] = []
  private active: TabId | null = null
  private nextId: TabId = 1
  private shieldEnabled = true
  private blockedTotal = 0
  private siteShield = new Map<TabId, { activeHere: boolean; blockedHere: number }>()
  private timers = new Map<TabId, ReturnType<typeof setInterval>>()
  private listeners = new Set<(event: CoreEvent) => void>()

  constructor() {
    this.openTab(HOME_URL)
  }

  public subscribe(listener: (event: CoreEvent) => void): () => void {
    this.listeners.add(listener)
    queueMicrotask(() => {
      listener({ kind: 'tabsChanged', tabs: this.tabs, active: this.active })
      if (this.active !== null) this.emitShield(this.active)
    })
    return () => {
      this.listeners.delete(listener)
    }
  }

  public send(request: UiRequest): void {
    if (!this.handleTabRequest(request)) this.handleShieldRequest(request)
  }

  private handleTabRequest(request: UiRequest): boolean {
    switch (request.kind) {
      case 'newTab':
        this.openTab(request.url ?? 'about:blank')
        return true
      case 'closeTab':
        this.closeTab(request.id)
        return true
      case 'selectTab':
        this.active = request.id
        this.emitTabs()
        this.emitShield(request.id)
        return true
      case 'navigate':
        this.navigate(request.id, toUrl(request.input))
        return true
      case 'goBack':
        this.patch(request.id, { canGoForward: true })
        this.startLoading(request.id)
        return true
      case 'goForward':
        this.patch(request.id, { canGoBack: true })
        this.startLoading(request.id)
        return true
      case 'reload':
        this.startLoading(request.id)
        return true
      case 'stop':
        this.finishLoading(request.id)
        return true
      default:
        return false
    }
  }

  private handleShieldRequest(request: UiRequest): void {
    switch (request.kind) {
      case 'setShieldEnabled':
        this.shieldEnabled = request.enabled
        for (const tab of this.tabs) this.emitShield(tab.id)
        return
      case 'toggleShieldForSite': {
        const site = this.siteOf(request.id)
        site.activeHere = !site.activeHere
        this.emitShield(request.id)
        return
      }
      case 'refreshFilterLists':
        this.emit({ kind: 'filterListsRefreshed', count: 7 })
        this.emit({ kind: 'notice', level: 'info', message: 'Listes de filtres à jour.' })
        return
      case 'openDevTools':
        this.emit({ kind: 'notice', level: 'info', message: 'DevTools indisponibles en développement.' })
        return
      default:
        return
    }
  }

  private openTab(url: string): void {
    const id = this.nextId++
    this.tabs = [...this.tabs, blankTab(id, url)]
    this.siteShield.set(id, { activeHere: true, blockedHere: 0 })
    this.active = id
    this.emitTabs()
    if (url !== 'about:blank') this.startLoading(id)
  }

  private closeTab(id: TabId): void {
    this.stopTimer(id)
    const index = this.tabs.findIndex((tab) => tab.id === id)
    this.tabs = this.tabs.filter((tab) => tab.id !== id)
    this.siteShield.delete(id)
    if (this.active === id) {
      const neighbour = this.tabs[Math.min(index, this.tabs.length - 1)]
      this.active = neighbour?.id ?? null
    }
    this.emitTabs()
    if (this.active !== null) this.emitShield(this.active)
  }

  private navigate(id: TabId, url: string): void {
    const site = this.siteOf(id)
    site.blockedHere = 0
    this.patch(id, { url, title: fallbackTitle(url), canGoBack: true, favicon: null })
    this.startLoading(id)
  }

  private startLoading(id: TabId): void {
    this.stopTimer(id)
    this.patch(id, { loading: true, progress: 0.04 })
    let tick = 0
    const timer = setInterval(() => {
      tick += 1
      if (tick >= LOAD_STEPS) {
        this.finishLoading(id)
        return
      }
      this.patch(id, { progress: tick / LOAD_STEPS })
      this.countBlocked(id, tick)
    }, LOAD_TICK_MS)
    this.timers.set(id, timer)
  }

  private finishLoading(id: TabId): void {
    this.stopTimer(id)
    const tab = this.tabs.find((candidate) => candidate.id === id)
    if (tab === undefined) return
    this.patch(id, { loading: false, progress: 1, title: fallbackTitle(tab.url) })
  }

  private countBlocked(id: TabId, tick: number): void {
    const site = this.siteOf(id)
    if (!this.shieldEnabled || !site.activeHere || tick % 2 !== 0) return
    const found = 1 + (tick % 3)
    site.blockedHere += found
    this.blockedTotal += found
    this.emitShield(id)
  }

  private stopTimer(id: TabId): void {
    const timer = this.timers.get(id)
    if (timer !== undefined) clearInterval(timer)
    this.timers.delete(id)
  }

  private siteOf(id: TabId): { activeHere: boolean; blockedHere: number } {
    const existing = this.siteShield.get(id)
    if (existing !== undefined) return existing
    const created = { activeHere: true, blockedHere: 0 }
    this.siteShield.set(id, created)
    return created
  }

  private patch(id: TabId, change: Partial<TabView>): void {
    const current = this.tabs.find((tab) => tab.id === id)
    if (current === undefined) return
    const updated: TabView = { ...current, ...change }
    this.tabs = this.tabs.map((tab) => (tab.id === id ? updated : tab))
    this.emit({ kind: 'tabUpdated', tab: updated })
  }

  private emitTabs(): void {
    this.emit({ kind: 'tabsChanged', tabs: this.tabs, active: this.active })
  }

  private emitShield(id: TabId): void {
    const site = this.siteOf(id)
    const state: ShieldView = {
      enabled: this.shieldEnabled,
      activeHere: this.shieldEnabled && site.activeHere,
      blockedHere: site.blockedHere,
      blockedTotal: this.blockedTotal,
    }
    this.emit({ kind: 'shieldUpdated', id, state })
  }

  private emit(event: CoreEvent): void {
    for (const listener of this.listeners) listener(event)
  }
}

/** Instancie le faux coeur. Appele uniquement quand `window.echo` est absent. */
export function createFakeCore(): CoreBridge {
  return new FakeCore()
}
