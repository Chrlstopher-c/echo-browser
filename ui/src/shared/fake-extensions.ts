// Responsabilite : volet extensions du faux coeur — installation lente, etats en attente, relance.
// Aucune logique reelle : il rejoue des echanges plausibles pour juger le panneau sans le navigateur.

import type { CoreEvent, ExtensionView, UiRequest } from './contract'

/** Duree simulee du telechargement depuis le catalogue. */
const DOWNLOAD_MS = 2600

/** Delai avant que la page simulee soit remplacee, comme le ferait la relance du processus. */
const RESTART_MS = 2800

const SEED: ExtensionView[] = [
  { id: 'ddkjiahejlhfcafbddmgiahcphecmpfh', name: 'uBlock Origin Lite', version: '2024.9.1', enabled: true,
    pending: false },
  { id: 'nngceckbapebfimnlniiiahkandclblb', name: 'Bitwarden', version: '2024.8.2', enabled: true, pending: false },
]

const CATALOGUE_NAMES = ['Dark Reader', 'Raindrop.io', 'Vimium', 'JSON Viewer', 'Wappalyzer']

type Emit = (event: CoreEvent) => void

/** Etat des extensions du faux coeur. Chaque attente est une minuterie unique, jamais une boucle. */
export class FakeExtensions {
  private items: ExtensionView[] = [...SEED]
  private restartPending = false

  constructor(private readonly emit: Emit) {}

  public snapshot(): CoreEvent {
    return { kind: 'extensionsChanged', extensions: this.items, restartPending: this.restartPending }
  }

  public handle(request: UiRequest): boolean {
    switch (request.kind) {
      case 'installExtension':
        this.install(request.source)
        return true
      case 'removeExtension':
        this.remove(request.id)
        return true
      case 'setExtensionEnabled':
        this.setEnabled(request.id, request.enabled)
        return true
      case 'restartBrowser':
        this.restart()
        return true
      default:
        return false
    }
  }

  private install(source: string): void {
    if (this.items.some((item) => item.id === source)) {
      this.emit({ kind: 'notice', level: 'warning', message: 'Cette extension est déjà installée.' })
      return
    }
    this.after(DOWNLOAD_MS, () => {
      const index = this.items.length % CATALOGUE_NAMES.length
      const name = CATALOGUE_NAMES[index] ?? 'Extension'
      this.items = [...this.items, { id: source, name, version: '1.0.0', enabled: true, pending: true }]
      this.restartPending = true
      this.emit(this.snapshot())
      this.emit({ kind: 'notice', level: 'info', message: `${name} installée.` })
    })
  }

  private remove(id: string): void {
    const gone = this.items.find((item) => item.id === id)
    if (gone === undefined) return
    this.items = this.items.filter((item) => item.id !== id)
    this.restartPending = true
    this.emit(this.snapshot())
    this.emit({ kind: 'notice', level: 'info', message: `${gone.name} retirée.` })
  }

  private setEnabled(id: string, enabled: boolean): void {
    this.items = this.items.map((item) => (item.id === id ? { ...item, enabled, pending: true } : item))
    this.restartPending = true
    this.emit(this.snapshot())
  }

  private restart(): void {
    this.emit({ kind: 'restarting', reason: 'Application des changements d’extensions.' })
    this.after(RESTART_MS, () => {
      // Le vrai coeur remplace le processus ; en developpement, recharger la page en tient lieu.
      window.location.reload()
    })
  }

  private after(delay: number, run: () => void): void {
    setTimeout(run, delay)
  }
}
