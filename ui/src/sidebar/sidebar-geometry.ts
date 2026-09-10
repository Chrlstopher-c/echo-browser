// Responsabilite : geometrie de la barre laterale — largeurs reclamees au coeur, rythme des transitions.

/** Barre depliee. */
export const SIDEBAR_WIDTH = 240

/** Rail replie : pastilles et favicons seuls. */
export const RAIL_WIDTH = 56

/** Marge entre la barre et le cadre de la page, en mode simule. Le coeur pose la sienne. */
export const STAGE_GUTTER = 8

export const CHROME_EASE: [number, number, number, number] = [0.22, 0.61, 0.36, 1]

/** Duree du repli et du depli, en secondes. */
export const COLLAPSE_SECONDS = 0.22

export function widthFor(collapsed: boolean): number {
  return collapsed ? RAIL_WIDTH : SIDEBAR_WIDTH
}
