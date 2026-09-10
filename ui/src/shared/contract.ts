// Responsabilite : miroir TypeScript du contrat d'echange avec le coeur Rust.
// Jumeau de core/contract/src/lib.rs — les deux se modifient ensemble, jamais l'un sans l'autre.

export type TabId = number

/** Ce que l'interface demande au coeur. */
export type UiRequest =
  | { kind: 'newTab'; url?: string }
  | { kind: 'closeTab'; id: TabId }
  | { kind: 'selectTab'; id: TabId }
  | { kind: 'navigate'; id: TabId; input: string }
  | { kind: 'goBack'; id: TabId }
  | { kind: 'goForward'; id: TabId }
  | { kind: 'reload'; id: TabId; bypassCache: boolean }
  | { kind: 'stop'; id: TabId }
  | { kind: 'toggleShieldForSite'; id: TabId }
  | { kind: 'setShieldEnabled'; enabled: boolean }
  | { kind: 'refreshFilterLists'; force: boolean }
  | { kind: 'openDevTools'; id: TabId }
  /** L'interface reclame une largeur : le coeur repositionne la vue du contenu a sa droite. */
  | { kind: 'setChromeWidth'; pixels: number }
  /** Replie ou deplie la barre laterale. */
  | { kind: 'setSidebarCollapsed'; collapsed: boolean }
  /** Teinte dominante de l'espace courant, appliquee au cadre autour de la page. */
  | { kind: 'setAccent'; color: string }

/** Ce que le coeur renvoie a l'interface. */
export type CoreEvent =
  | { kind: 'tabsChanged'; tabs: TabView[]; active: TabId | null }
  | { kind: 'tabUpdated'; tab: TabView }
  | { kind: 'shieldUpdated'; id: TabId; state: ShieldView }
  | { kind: 'filterListsRefreshed'; count: number }
  | { kind: 'notice'; level: NoticeLevel; message: string }

/** L'etat d'un onglet tel que l'interface l'affiche. */
export interface TabView {
  id: TabId
  title: string
  url: string
  loading: boolean
  progress: number
  canGoBack: boolean
  canGoForward: boolean
  favicon: string | null
  /** Etat de la connexion, tel que le coeur le connait. */
  security: Security
}

export type Security = 'secure' | 'mixed' | 'invalid' | 'insecure' | 'local'

/** L'etat du bouclier pour l'onglet courant. */
export interface ShieldView {
  /** Bouclier actif globalement. */
  enabled: boolean
  /** Bouclier actif sur ce site precis. */
  activeHere: boolean
  blockedHere: number
  blockedTotal: number
}

export type NoticeLevel = 'info' | 'warning' | 'error'

/** Pont expose par le coeur dans la page d'interface. */
export interface CoreBridge {
  send(request: UiRequest): void
  subscribe(listener: (event: CoreEvent) => void): () => void
}

declare global {
  interface Window {
    echo?: CoreBridge
  }
}
