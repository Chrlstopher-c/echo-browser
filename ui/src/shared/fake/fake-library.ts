// Responsabilite : volet bibliotheque du faux coeur — favoris et historique, avec recherche.

import type { BookmarkView, CoreEvent, HistoryEntryView, TabView, UiRequest } from '../contract'

const HOUR = 3_600_000
const DAY = 24 * HOUR

/** L'historique livre au plus ce nombre d'entrees ; `total` dit combien existent au-dela. */
const PAGE = 60

const SEED_BOOKMARKS: BookmarkView[] = [
  { url: 'https://developer.mozilla.org/', title: 'MDN Web Docs', favicon: null, addedAt: Date.now() - 12 * DAY },
  { url: 'https://github.com/', title: 'GitHub', favicon: null, addedAt: Date.now() - 9 * DAY },
  { url: 'https://news.ycombinator.com/', title: 'Hacker News', favicon: null, addedAt: Date.now() - 2 * DAY },
  { url: 'https://www.rust-lang.org/', title: 'Rust Programming Language', favicon: null, addedAt: Date.now() - DAY },
]

function seedHistory(now: number): HistoryEntryView[] {
  const rows: Array<[string, string, number, number]> = [
    ['https://github.com/', 'GitHub · Where the world builds software', 0.4 * HOUR, 41],
    ['https://developer.mozilla.org/fr/docs/Web/CSS', 'CSS : Feuilles de style en cascade | MDN', 1.2 * HOUR, 7],
    ['https://www.youtube.com/', 'YouTube', 2.6 * HOUR, 19],
    ['https://news.ycombinator.com/', 'Hacker News', 5 * HOUR, 63],
    ['https://crates.io/crates/tokio', 'tokio — crates.io', DAY + 1.5 * HOUR, 3],
    ['https://www.rust-lang.org/learn', 'Learn Rust', DAY + 3 * HOUR, 2],
    ['https://tauri.app/', 'Tauri 2.0 | Build smaller, faster apps', DAY + 6 * HOUR, 5],
    ['https://fr.wikipedia.org/wiki/Chromium', 'Chromium — Wikipédia', 2 * DAY + 2 * HOUR, 1],
    ['https://doc.rust-lang.org/book/', 'The Rust Programming Language', 3 * DAY + 4 * HOUR, 11],
    ['https://www.figma.com/', 'Figma', 4 * DAY + HOUR, 8],
  ]
  return rows.map(([url, title, ago, visits]) => ({ url, title, favicon: null, visitedAt: Math.floor((now - ago) / 1000), visits }))
}

type Emit = (event: CoreEvent) => void

export class FakeLibrary {
  private bookmarks: BookmarkView[] = [...SEED_BOOKMARKS]
  private history: HistoryEntryView[] = seedHistory(Date.now())
  private terms = ''

  constructor(
    private readonly emit: Emit,
    private readonly findTab: (id: number) => TabView | undefined,
  ) {}

  public snapshotBookmarks(): CoreEvent {
    return { kind: 'bookmarksChanged', bookmarks: this.bookmarks }
  }

  public snapshotHistory(): CoreEvent {
    const matching = this.matching()
    return { kind: 'historyChanged', entries: matching.slice(0, PAGE), total: matching.length }
  }

  public handle(request: UiRequest): boolean {
    switch (request.kind) {
      case 'addBookmark':
        this.addBookmark(request.id)
        return true
      case 'removeBookmark':
        this.bookmarks = this.bookmarks.filter((item) => item.url !== request.url)
        this.emit(this.snapshotBookmarks())
        return true
      case 'moveBookmark':
        this.moveBookmark(request.url, request.to)
        return true
      case 'removeHistoryEntry':
        this.history = this.history.filter(
          (entry) => !(entry.url === request.url && entry.visitedAt === request.visitedAt),
        )
        this.emit(this.snapshotHistory())
        return true
      case 'clearHistory':
        this.history = []
        this.emit(this.snapshotHistory())
        this.emit({ kind: 'notice', level: 'info', message: 'Historique effacé.' })
        return true
      case 'searchHistory':
        this.terms = request.terms.trim().toLowerCase()
        this.emit(this.snapshotHistory())
        return true
      default:
        return false
    }
  }

  /** Une page chargee devient une visite : l'entree remonte en tete, son compteur grandit. */
  public recordVisit(tab: TabView): void {
    if (tab.url === 'about:blank') return
    const previous = this.history.find((entry) => entry.url === tab.url)
    const visits = (previous?.visits ?? 0) + 1
    const entry: HistoryEntryView = {
      url: tab.url, title: tab.title, favicon: tab.favicon, visitedAt: Date.now(), visits,
    }
    this.history = [entry, ...this.history.filter((item) => item.url !== tab.url)]
    this.emit(this.snapshotHistory())
  }

  private matching(): HistoryEntryView[] {
    if (this.terms.length === 0) return this.history
    return this.history.filter(
      (entry) => entry.title.toLowerCase().includes(this.terms) || entry.url.toLowerCase().includes(this.terms),
    )
  }

  private addBookmark(id: number): void {
    const tab = this.findTab(id)
    if (tab === undefined) return
    if (this.bookmarks.some((item) => item.url === tab.url)) {
      this.emit({ kind: 'notice', level: 'info', message: 'Déjà dans les favoris.' })
      return
    }
    const added: BookmarkView = { url: tab.url, title: tab.title, favicon: tab.favicon, addedAt: Date.now() }
    this.bookmarks = [...this.bookmarks, added]
    this.emit(this.snapshotBookmarks())
    this.emit({ kind: 'notice', level: 'info', message: 'Ajouté aux favoris.' })
  }

  private moveBookmark(url: string, to: number): void {
    const from = this.bookmarks.findIndex((item) => item.url === url)
    if (from === -1) return
    const next = [...this.bookmarks]
    const [moved] = next.splice(from, 1)
    if (moved === undefined) return
    next.splice(Math.min(Math.max(to, 0), next.length), 0, moved)
    this.bookmarks = next
    this.emit(this.snapshotBookmarks())
  }
}
