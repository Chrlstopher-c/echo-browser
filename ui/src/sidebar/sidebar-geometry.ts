// Responsabilite : geometrie de la barre laterale — largeurs reclamees au coeur.

/** Barre depliee. */
export const SIDEBAR_WIDTH = 248

/** Marge entre la barre et le cadre de la page, en mode simule. Le coeur pose la sienne. */
export const STAGE_GUTTER = 8

/** Repliee ou non, la barre garde sa largeur : repliee, elle flotte par-dessus la page. */
export function widthFor(): number {
  return SIDEBAR_WIDTH
}
