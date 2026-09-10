// Responsabilite : lecture du modele d'extension — libelles et derivations, sans etat ni rendu.

import type { ExtensionView } from '../shared/contract'

/** Un changement d'extension ne prend effet qu'apres relance : Chromium ne les charge qu'au demarrage. */
export const RESTART_NOTICE = 'Des changements attendent la relance du navigateur.'

export const RESTART_HINT = 'Vos onglets seront retrouvés.'

/** Adresse du catalogue : le coeur l'ouvre quand on n'a pas d'extension precise en tete. */
export const STORE_URL = 'https://chromewebstore.google.com/'

/** Nombre d'extensions dont l'etat affiche ne correspond pas encore a ce qui tourne. */
export function pendingCount(extensions: ExtensionView[]): number {
  return extensions.filter((item) => item.pending).length
}

export function extensionStatus(item: ExtensionView): string {
  if (item.version === '') return 'Installation à la relance'
  if (item.pending) return item.enabled ? 'Activation à la relance' : 'Désactivation à la relance'
  return item.enabled ? 'Active' : 'Désactivée'
}

/** Les permissions les plus parlantes, mises en francais. Le reste garde son nom technique. */
const PERMISSION_LABELS: Record<string, string> = {
  activeTab: 'Onglet actif',
  alarms: 'Minuteries',
  bookmarks: 'Favoris',
  clipboardRead: 'Lire le presse-papiers',
  clipboardWrite: 'Écrire dans le presse-papiers',
  contextMenus: 'Menu contextuel',
  cookies: 'Cookies',
  downloads: 'Téléchargements',
  history: 'Historique',
  identity: 'Identité',
  management: 'Gestion des extensions',
  nativeMessaging: 'Applications de la machine',
  notifications: 'Notifications',
  offscreen: 'Traitement en arrière-plan',
  privacy: 'Réglages de confidentialité',
  scripting: 'Exécution de scripts dans les pages',
  storage: 'Stockage local',
  tabs: 'Onglets',
  unlimitedStorage: 'Stockage sans limite',
  webNavigation: 'Navigation',
  webRequest: 'Requêtes réseau',
  '<all_urls>': 'Tous les sites',
}

export function permissionLabel(permission: string): string {
  const known = PERMISSION_LABELS[permission]
  if (known !== undefined) return known
  if (permission.includes('://')) return permission
  return permission
}
