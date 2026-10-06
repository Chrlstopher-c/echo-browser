// Responsabilite : faux coeur en memoire pour le developpement de l'interface — assemble les volets
// (onglets, bouclier, extensions, bibliotheque, telechargements, reglages) et simule le clavier
// et le plein ecran. Aucune logique metier reelle : il rejoue des evenements plausibles.

import type { CoreBridge, CoreEvent, TabId, UiRequest } from '../contract'
import { FakeDownloads } from './fake-downloads'
import { FakeExtensions } from './fake-extensions'
import { FakeLibrary } from './fake-library'
import { FakeSettings } from './fake-settings'
import { FakeShield } from './fake-shield'
import { FakeTabs } from './fake-tabs'

/** Onglets ouverts au demarrage, pour juger la barre avec du contenu. Le premier est epingle. */
const SEED_URLS = [
  'https://www.google.com/',
  'https://github.com/',
  'https://developer.mozilla.org/',
  'https://www.youtube.com/',
  'http://neverssl.com/',
  'https://news.ycombinator.com/',
]

/** Leviers que seule la scene de developpement actionne, pour juger chaque etat sans le vrai navigateur. */
export interface FakeControls {
  toggleAudible: (id: TabId) => void
  toggleAsleep: (id: TabId) => void
  startDownload: () => void
  enterFullscreen: () => void
}

export interface FakeCoreHandle {
  bridge: CoreBridge
  controls: FakeControls
}

class FakeCore implements CoreBridge {
  private listeners = new Set<(event: CoreEvent) => void>()
  private fullscreen = false
  private readonly emit = (event: CoreEvent): void => {
    for (const listener of this.listeners) listener(event)
  }
  private readonly shield = new FakeShield(this.emit)
  private readonly extensions = new FakeExtensions(this.emit)
  private readonly settings = new FakeSettings(this.emit)
  private readonly downloads = new FakeDownloads(this.emit)
  private readonly tabs = new FakeTabs(this.emit, {
    onLoaded: (tab) => this.library.recordVisit(tab),
    onLoadTick: (id, tick) => this.shield.countBlocked(id, tick),
  })
  private readonly library = new FakeLibrary(this.emit, (id) => this.tabs.find(id))

  constructor() {
    for (const url of SEED_URLS) this.tabs.open(url, { silent: true })
    const first = this.tabs.all[0]
    if (first !== undefined) this.tabs.pin(first.id, true)
    const second = this.tabs.all[1]
    if (second !== undefined) this.tabs.select(second.id)
    const fourth = this.tabs.all[3]
    if (fourth !== undefined) this.tabs.setAudible(fourth.id, true)
    const last = this.tabs.all.at(-1)
    if (last !== undefined) this.tabs.setAsleep(last.id, true)
    window.addEventListener('keydown', this.onKeyDown)
  }

  public readonly controls: FakeControls = {
    toggleAudible: (id) => {
      const tab = this.tabs.find(id)
      if (tab !== undefined) this.tabs.setAudible(id, !tab.audible)
    },
    toggleAsleep: (id) => {
      const tab = this.tabs.find(id)
      if (tab !== undefined) this.tabs.setAsleep(id, !tab.asleep)
    },
    startDownload: () => this.downloads.start(),
    enterFullscreen: () => this.setFullscreen(true),
  }

  public subscribe(listener: (event: CoreEvent) => void): () => void {
    this.listeners.add(listener)
    queueMicrotask(() => {
      listener(this.tabs.snapshot())
      listener(this.shield.snapshotLists())
      listener(this.extensions.snapshot())
      listener(this.library.snapshotBookmarks())
      listener(this.library.snapshotHistory())
      listener(this.downloads.snapshot())
      listener(this.settings.snapshot())
      const active = this.tabs.activeId
      if (active !== null) this.shield.emitShield(active)
    })
    return () => {
      this.listeners.delete(listener)
    }
  }

  public send(request: UiRequest): void {
    if (this.handleTabRequest(request)) return
    if (this.shield.handle(request)) return
    if (this.extensions.handle(request)) return
    if (this.library.handle(request)) return
    if (this.downloads.handle(request)) return
    if (this.settings.handle(request)) return
    this.handleMisc(request)
  }

  private handleTabRequest(request: UiRequest): boolean {
    switch (request.kind) {
      case 'newTab':
        this.tabs.setContainer(this.tabs.open(request.url ?? 'about:blank'), request.container ?? null)
        return true
      case 'setTabContainer':
        this.tabs.setContainer(request.id, request.container)
        return true
      case 'warmTab':
        this.tabs.setAsleep(request.id, false)
        return true
      case 'sleepTab':
        this.tabs.setAsleep(request.id, true)
        return true
      case 'closeTab':
        this.tabs.close(request.id)
        this.shield.forget(request.id)
        this.emitActiveShield()
        return true
      case 'selectTab':
        this.tabs.select(request.id)
        this.shield.emitShield(request.id)
        return true
      case 'moveTab':
        this.tabs.move(request.id, request.to)
        return true
      case 'pinTab':
        this.tabs.pin(request.id, request.pinned)
        return true
      case 'setTabFolder':
        this.tabs.setFolder(request.id, request.folder)
        return true
      case 'navigate':
        this.shield.resetSite(request.id)
        this.tabs.navigate(request.id, request.input)
        return true
      default:
        return this.handleLoadRequest(request)
    }
  }

  private handleLoadRequest(request: UiRequest): boolean {
    switch (request.kind) {
      case 'goBack':
        this.tabs.back(request.id)
        return true
      case 'goForward':
        this.tabs.forward(request.id)
        return true
      case 'reload':
        this.shield.resetSite(request.id)
        this.tabs.reload(request.id)
        return true
      case 'stop':
        this.tabs.stop(request.id)
        return true
      case 'setZoom':
        this.tabs.setZoom(request.id, request.factor)
        return true
      default:
        return false
    }
  }

  private handleMisc(request: UiRequest): void {
    switch (request.kind) {
      case 'openDevTools':
        this.emit({ kind: 'notice', level: 'info', message: 'DevTools indisponibles en développement.' })
        return
      case 'openTerminal':
        this.emit({ kind: 'notice', level: 'info', message: 'Le terminal de Claude Code est absent en développement.' })
        return
      case 'exitFullscreen':
        this.setFullscreen(false)
        return
      default:
        return
    }
  }

  private setFullscreen(active: boolean): void {
    if (this.fullscreen === active) return
    this.fullscreen = active
    this.emit({ kind: 'fullscreenChanged', active })
  }

  private emitActiveShield(): void {
    const active = this.tabs.activeId
    if (active !== null) this.shield.emitShield(active)
  }

  /** Le vrai coeur recoit ces raccourcis de Chromium ; ici la page les capte a sa place. */
  private readonly onKeyDown = (event: KeyboardEvent): void => {
    if (event.key === 'l' && (event.ctrlKey || event.metaKey)) {
      event.preventDefault()
      this.emit({ kind: 'focusAddressRequested' })
      return
    }
    if (event.key === 'F11') {
      event.preventDefault()
      this.setFullscreen(!this.fullscreen)
      return
    }
    if (event.key === 'Escape' && this.fullscreen) this.setFullscreen(false)
  }
}

/** Instancie le faux coeur. Appele uniquement quand `window.echo` est absent. */
export function createFakeCore(): FakeCoreHandle {
  const core = new FakeCore()
  return { bridge: core, controls: core.controls }
}
