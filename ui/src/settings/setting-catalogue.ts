// Responsabilite : habillage des reglages — pour chaque cle que le coeur livre, un groupe, un libelle
// en francais, une explication, et la forme du controle. Une cle inconnue tombe dans « Autres ».

import type { SettingValue } from '../shared/contract'

export type SettingGroupId = 'navigation' | 'privacy' | 'page' | 'tabs' | 'downloads' | 'system' | 'other'

export interface SettingGroup {
  id: SettingGroupId
  title: string
}

export const SETTING_GROUPS: SettingGroup[] = [
  { id: 'navigation', title: 'Navigation' },
  { id: 'privacy', title: 'Vie privée' },
  { id: 'page', title: 'Affichage des pages' },
  { id: 'tabs', title: 'Onglets' },
  { id: 'downloads', title: 'Téléchargements' },
  { id: 'system', title: 'Système' },
  { id: 'other', title: 'Autres' },
]

export interface NumberShape {
  min: number
  max: number
  step: number
  unit: string
}

export interface SettingDefinition {
  group: SettingGroupId
  label: string
  /** Ce que le reglage fait, en une phrase. */
  detail: string
  /** Pour un nombre : bornes, pas, unite. */
  number?: NumberShape
  /** Pour un texte : exemple affiche quand le champ est vide, et police a chasse fixe. */
  placeholder?: string
  mono?: boolean
}

export const SETTING_CATALOGUE: Record<string, SettingDefinition> = {
  'search.engine': {
    group: 'navigation',
    label: 'Moteur de recherche',
    detail: 'Adresse utilisée quand la saisie n’est pas une URL. « %s » est remplacé par les termes.',
    placeholder: 'https://…/search?q=%s',
    mono: true,
  },
  'home.url': {
    group: 'navigation',
    label: 'Page d’accueil',
    detail: 'Page ouverte dans chaque nouvel onglet.',
    placeholder: 'https://',
    mono: true,
  },
  'session.restore': {
    group: 'navigation',
    label: 'Retrouver la session',
    detail: 'Rouvre les onglets de la dernière session au démarrage.',
  },
  'newtab.focusAddress': {
    group: 'navigation',
    label: 'Curseur dans l’adresse',
    detail: 'Un nouvel onglet place le curseur dans le champ d’adresse, prêt à taper.',
  },
  'privacy.doNotTrack': {
    group: 'privacy',
    label: 'Demander à ne pas être suivi',
    detail: 'Envoie l’en-tête « Do Not Track » aux sites. Ils restent libres de l’ignorer.',
  },
  'privacy.thirdPartyCookies': {
    group: 'privacy',
    label: 'Cookies tiers',
    detail: 'Autorise les cookies posés par un autre domaine que celui de la page. Coupé, certains sites cassent.',
  },
  'privacy.clearOnExit': {
    group: 'privacy',
    label: 'Effacer à la fermeture',
    detail: 'Supprime cookies et historique quand le navigateur se ferme.',
  },
  'privacy.httpsOnly': {
    group: 'privacy',
    label: 'HTTPS seulement',
    detail: 'Tente toujours la version chiffrée d’un site, et prévient si elle n’existe pas.',
  },
  'page.defaultZoom': {
    group: 'page',
    label: 'Zoom par défaut',
    detail: 'Taille des pages avant tout ajustement par onglet.',
    number: { min: 50, max: 200, step: 10, unit: '%' },
  },
  'page.minimumFontSize': {
    group: 'page',
    label: 'Taille minimale du texte',
    detail: 'Aucun texte n’est rendu plus petit que cette valeur. 0 laisse les sites décider.',
    number: { min: 0, max: 24, step: 1, unit: 'px' },
  },
  'page.smoothScrolling': {
    group: 'page',
    label: 'Défilement fluide',
    detail: 'Anime le défilement au clavier et à la molette.',
  },
  'tabs.sleepEnabled': {
    group: 'tabs',
    label: 'Endormir les onglets inactifs',
    detail: 'Libère la mémoire des onglets qu’on ne regarde plus. Ils se réveillent au clic.',
  },
  'tabs.sleepAfterMinutes': {
    group: 'tabs',
    label: 'Délai avant sommeil',
    detail: 'Temps d’inactivité avant qu’un onglet s’endorme.',
    number: { min: 5, max: 240, step: 5, unit: 'min' },
  },
  'tabs.neverSleep': {
    group: 'tabs',
    label: 'Sites qui ne dorment jamais',
    detail: 'Messageries et courrier, séparés par des virgules : leurs onglets restent éveillés.',
    mono: true,
  },
  'tabs.confirmCloseMany': {
    group: 'tabs',
    label: 'Confirmer la fermeture',
    detail: 'Demande confirmation avant de fermer une fenêtre à plusieurs onglets.',
  },
  'downloads.directory': {
    group: 'downloads',
    label: 'Dossier de réception',
    detail: 'Où les fichiers téléchargés sont écrits.',
    placeholder: '/home/…/Téléchargements',
    mono: true,
  },
  'downloads.askWhere': {
    group: 'downloads',
    label: 'Demander où enregistrer',
    detail: 'Ouvre un sélecteur de dossier à chaque téléchargement.',
  },
  'system.hardwareAcceleration': {
    group: 'system',
    label: 'Accélération matérielle',
    detail: 'Confie le rendu à la carte graphique. À couper si l’affichage se corrompt.',
  },
  'system.devTools': {
    group: 'system',
    label: 'Outils de développement',
    detail: 'Autorise l’inspecteur de page (F12).',
  },
  'experimental.webgpu': {
    group: 'system',
    label: 'WebGPU',
    detail: 'Active l’API graphique expérimentale. Peut rendre certaines pages instables.',
  },
}

/** Libelle de repli pour une cle que le catalogue ne connait pas : « privacy.fooBar » → « Foo bar ». */
export function fallbackLabel(key: string): string {
  const tail = key.split('.').at(-1) ?? key
  const spaced = tail.replace(/([a-z])([A-Z])/g, '$1 $2').toLowerCase()
  return spaced.charAt(0).toUpperCase() + spaced.slice(1)
}

export function definitionOf(key: string, value: SettingValue): SettingDefinition {
  const known = SETTING_CATALOGUE[key]
  if (known !== undefined) return known
  const fallback: SettingDefinition = { group: 'other', label: fallbackLabel(key), detail: key, mono: true }
  return value.type === 'number' ? { ...fallback, number: { min: 0, max: 10_000, step: 1, unit: '' } } : fallback
}
