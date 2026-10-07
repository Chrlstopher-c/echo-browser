// Responsabilite : volet onglets du faux coeur — ouverture, navigation, chargement simule, epinglage,
// deplacement, zoom, son et sommeil. Aucune logique reelle : des evenements plausibles, rien de plus.

import type { CoreEvent, TabId, TabView } from '../contract'
import { fallbackTitle, looksLikeUrl, securityOf } from '../url-shape'

const LOAD_STEPS = 18
const LOAD_TICK_MS = 95
const SEARCH_PREFIX = 'https://www.google.com/search?q='
const ZOOM_MIN = 0.5
const ZOOM_MAX = 3

/** Titres plausibles pour les hotes de demarrage, pour juger la troncature et le rythme. */
const KNOWN_TITLES: Record<string, string> = {
  'anthropic.com': 'Anthropic',
  'github.com': 'GitHub · Where the world builds software',
  'neverssl.com': 'NeverSSL — Helping you get online',
  'developer.mozilla.org': 'MDN Web Docs',
  'www.youtube.com': 'YouTube',
  'news.ycombinator.com': 'Hacker News',
  'www.google.com': 'Google',
}

type Emit = (event: CoreEvent) => void

export interface TabHooks {
  /** Une page a fini de charger : l'historique s'en souvient. */
  onLoaded: (tab: TabView) => void
  /** Un pas de chargement : le bouclier compte ce qu'il bloque. */
  onLoadTick: (id: TabId, tick: number) => void
}

export function toUrl(input: string): string {
  const value = input.trim()
  if (!looksLikeUrl(value)) return `${SEARCH_PREFIX}${encodeURIComponent(value)}`
  return /^[a-z][a-z0-9+.-]*:/i.test(value) ? value : `https://${value}`
}

function titleFor(url: string): string {
  try {
    const host = new URL(url).host
    return KNOWN_TITLES[host] ?? fallbackTitle(url)
  } catch {
    return fallbackTitle(url)
  }
}

function blankTab(id: TabId, url: string): TabView {
  return {
    id,
    title: titleFor(url),
    url,
    loading: false,
    progress: 0,
    canGoBack: false,
    canGoForward: false,
    favicon: null,
    security: securityOf(url),
    pinned: false,
    folder: null,
    container: null,
    space: 'graphite',
    zoom: 1,
    audible: false,
    asleep: false,
  }
}

function clampZoom(factor: number): number {
  return Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, Math.round(factor * 100) / 100))
}

/** Onglets du faux coeur. Chaque chargement est une minuterie bornee par LOAD_STEPS. */
export class FakeTabs {
  private tabs: TabView[] = []
  private active: TabId | null = null
  private nextId: TabId = 1
  private timers = new Map<TabId, ReturnType<typeof setInterval>>()

  constructor(
    private readonly emit: Emit,
    private readonly hooks: TabHooks,
  ) {}

  public get all(): TabView[] {
    return this.tabs
  }

  public get activeId(): TabId | null {
    return this.active
  }

  public snapshot(): CoreEvent {
    return { kind: 'tabsChanged', tabs: this.tabs, active: this.active }
  }

  public find(id: TabId): TabView | undefined {
    return this.tabs.find((tab) => tab.id === id)
  }

  public open(url: string, options: { silent?: boolean } = {}): TabId {
    const id = this.nextId++
    this.tabs = [...this.tabs, blankTab(id, url)]
    this.active = id
    if (options.silent !== true) this.emitTabs()
    if (url !== 'about:blank') this.startLoading(id)
    return id
  }

  public close(id: TabId): void {
    this.stopTimer(id)
    const index = this.tabs.findIndex((tab) => tab.id === id)
    this.tabs = this.tabs.filter((tab) => tab.id !== id)
    if (this.active === id) {
      const neighbour = this.tabs[Math.min(index, this.tabs.length - 1)]
      this.active = neighbour?.id ?? null
    }
    this.emitTabs()
  }

  public select(id: TabId): void {
    if (this.find(id) === undefined) return
    this.active = id
    this.patch(id, { asleep: false })
    this.emitTabs()
  }

  public move(id: TabId, to: number): void {
    const from = this.tabs.findIndex((tab) => tab.id === id)
    if (from === -1) return
    const next = [...this.tabs]
    const [moved] = next.splice(from, 1)
    if (moved === undefined) return
    next.splice(Math.min(Math.max(to, 0), next.length), 0, moved)
    this.tabs = next
    this.emitTabs()
  }

  public pin(id: TabId, pinned: boolean): void {
    this.patch(id, { pinned })
    this.emitTabs()
  }

  public setContainer(id: TabId, container: string | null): void {
    this.patch(id, { container })
    this.emitTabs()
  }

  public setFolder(id: TabId, folder: string | null): void {
    this.patch(id, { folder })
    this.emitTabs()
  }

  public navigate(id: TabId, input: string): void {
    const url = toUrl(input)
    this.patch(id, {
      url,
      title: titleFor(url),
      canGoBack: true,
      favicon: null,
      security: securityOf(url),
      asleep: false,
    })
    this.startLoading(id)
  }

  public back(id: TabId): void {
    this.patch(id, { canGoForward: true })
    this.startLoading(id)
  }

  public forward(id: TabId): void {
    this.patch(id, { canGoBack: true })
    this.startLoading(id)
  }

  public reload(id: TabId): void {
    this.patch(id, { asleep: false })
    this.startLoading(id)
  }

  public stop(id: TabId): void {
    this.stopTimer(id)
    this.patch(id, { loading: false, progress: 0 })
  }

  public setZoom(id: TabId, factor: number): void {
    this.patch(id, { zoom: clampZoom(factor) })
  }

  public setAudible(id: TabId, audible: boolean): void {
    this.patch(id, { audible })
  }

  public setAsleep(id: TabId, asleep: boolean): void {
    this.stopTimer(id)
    this.patch(id, { asleep, loading: false, audible: false })
  }

  public setFavicon(id: TabId, favicon: string | null): void {
    this.patch(id, { favicon })
  }

  private startLoading(id: TabId): void {
    this.stopTimer(id)
    this.patch(id, { loading: true, progress: 0.03 })
    let tick = 0
    const timer = setInterval(() => {
      tick += 1
      if (tick >= LOAD_STEPS) {
        this.finishLoading(id)
        return
      }
      this.patch(id, { progress: easeProgress(tick / LOAD_STEPS) })
      this.hooks.onLoadTick(id, tick)
    }, LOAD_TICK_MS)
    this.timers.set(id, timer)
  }

  private finishLoading(id: TabId): void {
    this.stopTimer(id)
    const tab = this.find(id)
    if (tab === undefined) return
    this.patch(id, { loading: false, progress: 1 })
    const loaded = this.find(id)
    if (loaded !== undefined) this.hooks.onLoaded(loaded)
  }

  private stopTimer(id: TabId): void {
    const timer = this.timers.get(id)
    if (timer !== undefined) clearInterval(timer)
    this.timers.delete(id)
  }

  private patch(id: TabId, change: Partial<TabView>): void {
    const current = this.find(id)
    if (current === undefined) return
    const updated: TabView = { ...current, ...change }
    this.tabs = this.tabs.map((tab) => (tab.id === id ? updated : tab))
    this.emit({ kind: 'tabUpdated', tab: updated })
  }

  private emitTabs(): void {
    this.emit(this.snapshot())
  }
}

/** Un vrai chargement va vite au debut et traine sur la fin : la courbe imite ce profil. */
function easeProgress(linear: number): number {
  return Math.min(0.96, 1 - Math.pow(1 - linear, 1.8))
}
