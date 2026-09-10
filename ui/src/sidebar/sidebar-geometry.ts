// Responsabilite : geometrie de la barre laterale — largeurs reclamees au coeur.

/** Barre depliee. */
export const SIDEBAR_WIDTH = 248

/** Rail replie : favicons seuls. */
export const RAIL_WIDTH = 56

/** Marge entre la barre et le cadre de la page, en mode simule. Le coeur pose la sienne. */
export const STAGE_GUTTER = 8

export function widthFor(collapsed: boolean): number {
  return collapsed ? RAIL_WIDTH : SIDEBAR_WIDTH
}
