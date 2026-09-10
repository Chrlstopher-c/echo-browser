// Responsabilite : lecture du modele d'extension — libelles et derivations, sans etat ni rendu.

import type { ExtensionView } from '../shared/contract'

/** Un changement d'extension ne prend effet qu'apres relance : Chromium ne les charge qu'au demarrage. */
export const RESTART_NOTICE = 'Des changements attendent la relance du navigateur.'

export const RESTART_HINT = 'Vos onglets seront retrouvés.'

/** Empreinte de l'inventaire : change des qu'une extension apparait, disparait ou bascule. */
export function inventorySignature(extensions: ExtensionView[]): string {
  return extensions.map((item) => `${item.id}:${item.version}:${item.enabled ? 1 : 0}`).join('|')
}

/** Nombre d'extensions dont l'etat affiche ne correspond pas encore a ce qui tourne. */
export function pendingCount(extensions: ExtensionView[]): number {
  return extensions.filter((item) => item.pending).length
}

export function extensionStatus(item: ExtensionView): string {
  if (item.pending) return item.enabled ? 'Activation à la relance' : 'Désactivation à la relance'
  return item.enabled ? 'Active' : 'Désactivée'
}
