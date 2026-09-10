// Responsabilite : identite et hauteur des panneaux du chrome.

import { CHROME_REST_HEIGHT } from '../shared/design/geometry'

export type PanelId = 'shield' | 'menu' | 'bookmarks' | 'history' | 'downloads' | 'extensions'

/** Hauteur du panneau seul, hors bande de chrome. */
export const PANEL_HEIGHT: Record<PanelId, number> = {
  shield: 296,
  menu: 214,
  bookmarks: 340,
  history: 340,
  downloads: 340,
  extensions: 324,
}

/** Hauteur totale a reclamer au coeur pour l'etat de panneau donne. */
export function chromeHeightFor(panel: PanelId | null): number {
  return panel === null ? CHROME_REST_HEIGHT : CHROME_REST_HEIGHT + PANEL_HEIGHT[panel]
}
