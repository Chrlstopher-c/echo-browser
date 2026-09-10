// Responsabilite : volet extensions du faux coeur — inventaire, bascule, retrait, relance.
// L'installation ouvre la fiche du catalogue : ici, on fait comme si Chromium l'avait acceptee.

import type { CoreEvent, ExtensionView, UiRequest } from '../contract'

/** Delai simule entre l'ouverture de la fiche et l'acceptation des permissions. */
const STORE_MS = 2200

/** Delai avant que la page simulee soit remplacee, comme le ferait la relance du processus. */
const RESTART_MS = 2800

const ICON_UBLOCK =
  'data:image/svg+xml;utf8,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" rx="7" ' +
      'fill="%23800000"/><path d="M16 5 7 8.5v7c0 5.2 3.8 9.4 9 11.5 5.2-2.1 9-6.3 9-11.5v-7z" fill="%23fff"/></svg>',
  )

const ICON_BITWARDEN =
  'data:image/svg+xml;utf8,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" rx="7" ' +
      'fill="%23175ddc"/><path d="M16 6 8 9v7c0 4.6 3.3 8.4 8 10 4.7-1.6 8-5.4 8-10V9z" fill="none" ' +
      'stroke="%23fff" stroke-width="2.4"/></svg>',
  )

const SEED: ExtensionView[] = [
  {
    id: 'ddkjiahejlhfcafbddmgiahcphecmpfh',
    name: 'uBlock Origin Lite',
    version: '2024.9.1',
    enabled: true,
    pending: false,
    removable: true,
    icon: ICON_UBLOCK,
    popup: 'chrome-extension://exemple/popup.html',
  },
  {
    id: 'nngceckbapebfimnlniiiahkandclblb',
    name: 'Bitwarden',
    version: '2024.8.2',
    enabled: true,
    pending: false,
    removable: true,
    icon: ICON_BITWARDEN,
    popup: 'chrome-extension://exemple/popup.html',
  },
  {
    id: 'echo-internal-reader',
    name: 'Lecteur PDF intégré',
    version: '1.2.0',
    enabled: true,
    pending: false,
    removable: false,
    icon: null,
    popup: 'chrome-extension://exemple/popup.html',
  },
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
      case 'openExtensionManager':
        this.emit({ kind: 'notice', level: 'info', message: 'Gestionnaire Chromium indisponible en développement.' })
        return true
      case 'restartBrowser':
        this.restart()
        return true
      default:
        return false
    }
  }

  private install(source: string): void {
    const id = /[a-p]{32}/.exec(source)?.[0] ?? null
    if (id === null) {
      this.emit({ kind: 'notice', level: 'info', message: 'Catalogue ouvert dans un onglet.' })
      return
    }
    if (this.items.some((item) => item.id === id)) {
      this.emit({ kind: 'notice', level: 'warning', message: 'Cette extension est déjà installée.' })
      return
    }
    this.emit({ kind: 'notice', level: 'info', message: 'Fiche ouverte : Chromium demande les permissions.' })
    setTimeout(() => {
      const name = CATALOGUE_NAMES[this.items.length % CATALOGUE_NAMES.length] ?? 'Extension'
      const added: ExtensionView = {
        id, name, version: '1.0.0', enabled: true, pending: true, removable: true, icon: null,
        popup: 'chrome-extension://exemple/popup.html',
      }
      this.items = [...this.items, added]
      this.restartPending = true
      this.emit(this.snapshot())
      this.emit({ kind: 'notice', level: 'info', message: `${name} installée.` })
    }, STORE_MS)
  }

  private remove(id: string): void {
    const gone = this.items.find((item) => item.id === id)
    if (gone === undefined || !gone.removable) return
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
    setTimeout(() => {
      // Le vrai coeur remplace le processus ; en developpement, recharger la page en tient lieu.
      window.location.reload()
    }, RESTART_MS)
  }
}
