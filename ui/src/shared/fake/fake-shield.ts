// Responsabilite : volet bouclier du faux coeur — compteurs par onglet, interrupteurs, listes de filtres.

import type { CoreEvent, FilterListView, ShieldView, TabId, UiRequest } from '../contract'

/** Duree simulee du rafraichissement des listes. */
const REFRESH_MS = 1400

const SEED_LISTS: FilterListView[] = [
  { id: 'easylist', title: 'EasyList', enabled: true, rules: 71_284 },
  { id: 'easyprivacy', title: 'EasyPrivacy', enabled: true, rules: 52_910 },
  { id: 'ublock-filters', title: 'uBlock filters', enabled: true, rules: 38_402 },
  { id: 'ublock-badware', title: 'Sites malveillants', enabled: true, rules: 9_118 },
  { id: 'easylist-fr', title: 'Liste FR', enabled: true, rules: 6_744 },
  { id: 'annoyances', title: 'Bannières de cookies', enabled: false, rules: null },
  { id: 'fanboy-social', title: 'Boutons sociaux', enabled: false, rules: null },
]

type Emit = (event: CoreEvent) => void

interface SiteShield {
  activeHere: boolean
  blockedHere: number
}

export class FakeShield {
  private enabled = true
  private blockedTotal = 1_284
  private sites = new Map<TabId, SiteShield>()
  private lists: FilterListView[] = [...SEED_LISTS]
  private refreshedAt: number | null = Math.floor(Date.now() / 1000) - 3 * 3600
  private refreshTimer: ReturnType<typeof setTimeout> | null = null

  constructor(private readonly emit: Emit) {}

  public snapshotLists(): CoreEvent {
    return { kind: 'filterListsChanged', lists: this.lists, refreshedAt: this.refreshedAt }
  }

  public handle(request: UiRequest): boolean {
    switch (request.kind) {
      case 'setShieldEnabled':
        this.enabled = request.enabled
        for (const id of this.sites.keys()) this.emitShield(id)
        return true
      case 'toggleShieldForSite': {
        const site = this.siteOf(request.id)
        site.activeHere = !site.activeHere
        this.emitShield(request.id)
        return true
      }
      case 'refreshFilterLists':
        this.refresh()
        return true
      case 'setFilterListEnabled':
        this.setListEnabled(request.id, request.enabled)
        return true
      default:
        return false
    }
  }

  public forget(id: TabId): void {
    this.sites.delete(id)
  }

  public resetSite(id: TabId): void {
    this.siteOf(id).blockedHere = 0
  }

  /** A chaque pas de chargement pair, quelques requetes sont « bloquees ». */
  public countBlocked(id: TabId, tick: number): void {
    const site = this.siteOf(id)
    if (!this.enabled || !site.activeHere || tick % 2 !== 0) return
    const found = 1 + (tick % 3)
    site.blockedHere += found
    this.blockedTotal += found
    this.emitShield(id)
  }

  public emitShield(id: TabId): void {
    const site = this.siteOf(id)
    const state: ShieldView = {
      enabled: this.enabled,
      activeHere: this.enabled && site.activeHere,
      blockedHere: site.blockedHere,
      blockedTotal: this.blockedTotal,
    }
    this.emit({ kind: 'shieldUpdated', id, state })
  }

  private siteOf(id: TabId): SiteShield {
    const existing = this.sites.get(id)
    if (existing !== undefined) return existing
    const created = { activeHere: true, blockedHere: 0 }
    this.sites.set(id, created)
    return created
  }

  private setListEnabled(id: string, enabled: boolean): void {
    this.lists = this.lists.map((list) => {
      if (list.id !== id) return list
      const rules = enabled ? (list.rules ?? 4_000 + (id.length * 977) % 9_000) : null
      return { ...list, enabled, rules }
    })
    this.emit(this.snapshotLists())
  }

  private refresh(): void {
    if (this.refreshTimer !== null) return
    this.refreshTimer = setTimeout(() => {
      this.refreshTimer = null
      this.refreshedAt = Math.floor(Date.now() / 1000)
      this.lists = this.lists.map((list) =>
        list.enabled && list.rules !== null ? { ...list, rules: list.rules + 12 } : list,
      )
      this.emit(this.snapshotLists())
      this.emit({ kind: 'notice', level: 'info', message: 'Listes de filtres à jour.', actions: [] })
    }, REFRESH_MS)
  }
}
