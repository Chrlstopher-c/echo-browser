// Responsabilite : volet telechargements du faux coeur — progression en direct, fin, annulation, oubli.

import type { CoreEvent, DownloadId, DownloadView, UiRequest } from '../contract'

const TICK_MS = 160
/** Octets recus par pas, module par un debit qui varie pour que la barre respire. */
const BASE_RATE = 1_150_000
const MAX_TICKS = 600

const SEED: DownloadView[] = [
  {
    id: 1,
    fileName: 'echo-browser-0.3.0-x86_64.AppImage',
    url: 'https://github.com/echo/browser/releases/download/v0.3.0/echo-browser.AppImage',
    path: '/home/chris/Téléchargements/echo-browser-0.3.0-x86_64.AppImage',
    received: 118_400_000,
    total: 118_400_000,
    state: 'complete',
    startedAt: Math.floor(Date.now() / 1000) - 2 * 3600,
  },
  {
    id: 2,
    fileName: 'rapport-q3.pdf',
    url: 'https://intranet.example.org/rapports/q3.pdf',
    path: null,
    received: 2_100_000,
    total: 6_400_000,
    state: 'failed',
    startedAt: Math.floor(Date.now() / 1000) - 40 * 60,
  },
]

/** Fichiers que la simulation propose quand on declenche un telechargement. */
const CATALOGUE: Array<{ fileName: string; total: number | null }> = [
  { fileName: 'ubuntu-24.04.1-desktop-amd64.iso', total: 6_110_000_000 },
  { fileName: 'photos-vacances.zip', total: 384_000_000 },
  { fileName: 'facture-2026-09.pdf', total: 412_000 },
  { fileName: 'archive-sans-taille.tar.gz', total: null },
]

type Emit = (event: CoreEvent) => void

export class FakeDownloads {
  private items: DownloadView[] = [...SEED]
  private nextId: DownloadId = 3
  private timers = new Map<DownloadId, ReturnType<typeof setInterval>>()
  private started = 0

  constructor(private readonly emit: Emit) {}

  public snapshot(): CoreEvent {
    return { kind: 'downloadsChanged', downloads: this.items }
  }

  public handle(request: UiRequest): boolean {
    switch (request.kind) {
      case 'openDownload':
        this.emit({ kind: 'notice', level: 'info', message: `Ouverture de ${this.nameOf(request.id)}.`, actions: [] })
        return true
      case 'revealDownload':
        this.emit({ kind: 'notice', level: 'info', message: 'Dossier ouvert dans le gestionnaire de fichiers.', actions: [] })
        return true
      case 'cancelDownload':
        this.cancel(request.id)
        return true
      case 'forgetDownload':
        this.stopTimer(request.id)
        this.items = this.items.filter((item) => item.id !== request.id)
        this.emit(this.snapshot())
        return true
      default:
        return false
    }
  }

  /** Levier de simulation : demarre un fichier du catalogue. */
  public start(): void {
    const pick = CATALOGUE[this.started % CATALOGUE.length]
    this.started += 1
    if (pick === undefined) return
    const id = this.nextId++
    const item: DownloadView = {
      id,
      fileName: pick.fileName,
      url: `https://files.example.org/${pick.fileName}`,
      path: null,
      received: 0,
      total: pick.total,
      state: 'running',
      startedAt: Math.floor(Date.now() / 1000),
    }
    this.items = [item, ...this.items]
    this.emit(this.snapshot())
    this.run(id, pick.total)
  }

  private run(id: DownloadId, total: number | null): void {
    let tick = 0
    const cap = total ?? 260_000_000
    const timer = setInterval(() => {
      tick += 1
      const current = this.items.find((item) => item.id === id)
      if (current === undefined || current.state !== 'running' || tick > MAX_TICKS) {
        this.stopTimer(id)
        return
      }
      const rate = BASE_RATE * (0.55 + 0.45 * Math.abs(Math.sin(tick / 6)))
      const received = Math.min(cap, current.received + Math.round(rate * (cap / 60_000_000 + 1)))
      const done = received >= cap
      this.patch(id, {
        received,
        state: done ? 'complete' : 'running',
        path: done ? `/home/chris/Téléchargements/${current.fileName}` : null,
      })
      if (done) this.stopTimer(id)
    }, TICK_MS)
    this.timers.set(id, timer)
  }

  private cancel(id: DownloadId): void {
    this.stopTimer(id)
    this.patch(id, { state: 'cancelled' })
  }

  private patch(id: DownloadId, change: Partial<DownloadView>): void {
    this.items = this.items.map((item) => (item.id === id ? { ...item, ...change } : item))
    this.emit(this.snapshot())
  }

  private stopTimer(id: DownloadId): void {
    const timer = this.timers.get(id)
    if (timer !== undefined) clearInterval(timer)
    this.timers.delete(id)
  }

  private nameOf(id: DownloadId): string {
    return this.items.find((item) => item.id === id)?.fileName ?? 'fichier'
  }
}
