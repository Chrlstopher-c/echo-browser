// Responsabilite : lecture du bouclier — mode affiche, libelles, mise en forme des compteurs.

import type { ShieldView } from '../shared/contract'

export type ShieldMode = 'active' | 'siteOff' | 'globalOff'

export function shieldModeOf(view: ShieldView): ShieldMode {
  if (!view.enabled) return 'globalOff'
  return view.activeHere ? 'active' : 'siteOff'
}

export const SHIELD_LABEL: Record<ShieldMode, string> = {
  active: 'Protection active',
  siteOff: 'Désactivé sur ce site',
  globalOff: 'Protection coupée',
}

/** Compteur lisible : espace fine insecable tous les trois chiffres. */
export function formatCount(value: number): string {
  return value.toLocaleString('fr-FR').replace(/ | /g, ' ')
}
