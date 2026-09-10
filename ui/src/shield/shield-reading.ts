// Responsabilite : lecture du bouclier — mode affiche, libelles.

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

export const SHIELD_DETAIL: Record<ShieldMode, string> = {
  active: 'Traqueurs et publicités filtrés ici.',
  siteOff: 'Aucun filtrage sur cette page.',
  globalOff: 'Aucun filtrage, sur aucun site.',
}
