// Responsabilite : habillage des reglages — pour chaque cle que le coeur livre, un groupe, un libelle
// en francais, une explication, et la forme du controle. Une cle inconnue n'est pas montree.

export type SettingGroupId = 'navigation' | 'privacy' | 'tabs' | 'downloads' | 'system'

export interface SettingGroup {
  id: SettingGroupId
  title: string
}

export const SETTING_GROUPS: SettingGroup[] = [
  { id: 'navigation', title: 'Navigation' },
  { id: 'privacy', title: 'Vie privée' },
  { id: 'tabs', title: 'Onglets' },
  { id: 'downloads', title: 'Téléchargements' },
  { id: 'system', title: 'Système' },
]

export interface NumberShape {
  min: number
  max: number
  step: number
  unit: string
}

export interface ChoiceOption {
  id: string
  label: string
}

export interface SettingDefinition {
  group: SettingGroupId
  label: string
  /** Ce que le reglage fait, en une phrase. */
  detail: string
  /** Pour un nombre : bornes, pas, unite. */
  number?: NumberShape
  /** Pour un texte a valeurs fermees : la liste des choix. */
  choices?: ChoiceOption[]
  /** Pour un texte libre : exemple affiche quand le champ est vide. */
  placeholder?: string
  /** Texte qui est une liste de sites separes par des virgules : affiche en pastilles. */
  sites?: boolean
}

/** Moteurs proposes ; les identifiants sont ceux du coeur (`core/shell/src/search.rs`). */
export const SEARCH_ENGINES: ChoiceOption[] = [
  { id: 'google', label: 'Google' },
  { id: 'duckduckgo', label: 'DuckDuckGo' },
  { id: 'qwant', label: 'Qwant' },
  { id: 'ecosia', label: 'Ecosia' },
  { id: 'bing', label: 'Bing' },
  { id: 'startpage', label: 'Startpage' },
  { id: 'brave', label: 'Brave Search' },
]

/** Seuls les reglages decrits ici sont montres : une cle interne du coeur n'apparait jamais dans la feuille. */
export const SETTING_CATALOGUE: Record<string, SettingDefinition> = {
  'search.engine': {
    group: 'navigation',
    label: 'Moteur de recherche',
    detail: 'Utilisé quand ce qui est tapé dans l’adresse n’est pas une adresse de site.',
    choices: SEARCH_ENGINES,
  },
  'search.suggest': {
    group: 'navigation',
    label: 'Suggestions du moteur',
    detail: 'Pendant la frappe, le moteur propose des recherches. Ce qui est tapé lui est alors envoyé.',
  },
  'session.restore': {
    group: 'navigation',
    label: 'Retrouver la session',
    detail: 'Rouvre les onglets de la dernière session au démarrage.',
  },
  'privacy.send_do_not_track': {
    group: 'privacy',
    label: 'Demander à ne pas être suivi',
    detail: 'Envoie aux sites les signaux « Do Not Track » et « Global Privacy Control ». Ils restent libres de les '
      + 'ignorer, mais la loi en impose certains.',
  },
  'privacy.clear_on_exit': {
    group: 'privacy',
    label: 'Tout effacer à la fermeture',
    detail: 'Historique, cookies et cache disparaissent quand Echo se ferme. Les comptes sont alors déconnectés.',
  },
  'signals.share': {
    group: 'privacy',
    label: 'Partager des signaux anonymes',
    detail: 'Chaque jour, des comptes de domaines visités et de traqueurs bloqués, sans compte ni identifiant, jamais '
      + 'd’adresse complète. Aide à repérer les sites malveillants. Coupé par défaut.',
  },
  'tabs.sleepEnabled': {
    group: 'tabs',
    label: 'Endormir les onglets inactifs',
    detail: 'Libère la mémoire des onglets qu’on ne regarde plus. Ils se réveillent au clic, à la même position.',
  },
  'tabs.sleepAfterMinutes': {
    group: 'tabs',
    label: 'Délai avant sommeil',
    detail: 'Temps d’inactivité avant qu’un onglet s’endorme. Plus court seulement si la mémoire de l’ordinateur '
      + 'vient à manquer.',
    number: { min: 5, max: 240, step: 5, unit: 'min' },
  },
  'tabs.neverSleep': {
    group: 'tabs',
    label: 'Sites qui ne dorment jamais',
    detail: 'Messageries et courrier : leurs onglets restent éveillés pour recevoir les messages.',
    sites: true,
  },
  'downloads.ask_location': {
    group: 'downloads',
    label: 'Demander où enregistrer',
    detail: 'Ouvre un sélecteur à chaque téléchargement. Sinon, tout va dans le dossier Téléchargements.',
  },
  'updates.auto': {
    group: 'system',
    label: 'Mises à jour automatiques',
    detail: 'Echo télécharge les nouvelles versions et propose de redémarrer pour les appliquer.',
  },
}

export function definitionOf(key: string): SettingDefinition | null {
  return SETTING_CATALOGUE[key] ?? null
}
