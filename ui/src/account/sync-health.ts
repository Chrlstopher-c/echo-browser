// Responsabilite : dire si la machine est a jour avec son compte — selon le mode, l'anciennete de la derniere synchro,
// une erreur, ou des modifications qui attendent. Partage par l'alerte de la barre et la section Compte.

import type { AccountView } from '../shared/contract'

/** Au-dela, une synchro automatique aurait du passer : la machine n'est plus consideree a jour. */
const STALE_MINUTES: Record<'realtime' | 'auto', number> = { realtime: 5, auto: 20 }

export interface SyncWarning {
  message: string
  action: string
}

function minutesSince(seconds: number | null): number {
  return seconds === null ? Infinity : (Date.now() / 1000 - seconds) / 60
}

export function syncWarning(account: AccountView | null): SyncWarning | null {
  if (account === null || !account.available || account.email === null || account.busy) return null
  if (account.error !== null) return { message: `Pas synchronisé : ${account.error}.`, action: 'Réessayer' }
  if (account.mode === 'manual') {
    return account.pending ? { message: 'Des modifications ne sont pas synchronisées.', action: 'Synchroniser' } : null
  }
  const minutes = minutesSince(account.lastSync)
  if (minutes <= STALE_MINUTES[account.mode]) return null
  const since = minutes === Infinity ? 'jamais synchronisé' : `pas synchronisé depuis ${Math.round(minutes)} min`
  return { message: `Attention : ${since}.`, action: 'Synchroniser' }
}
