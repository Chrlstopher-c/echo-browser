// Responsabilite : modele des essentiels — onglets epingles, tenus par l'interface. Le contrat ne
// connait pas l'epinglage : un essentiel est un hote, retrouve parmi les onglets ouverts.

import type { TabView } from '../shared/contract'
import { readUrl } from '../shared/url-shape'

export interface Essential {
  /** Hote sans « www. », cle d'identite. */
  host: string
  url: string
  title: string
  favicon: string | null
}

/** Colonnes de la grille de pastilles. */
export const ESSENTIALS_COLUMNS = 2

/** Essentiels proposes en simulation quand rien n'est encore epingle. */
export const SEED_ESSENTIALS: Essential[] = [
  { host: 'anthropic.com', url: 'https://anthropic.com/', title: 'Anthropic', favicon: null },
  { host: 'github.com', url: 'https://github.com/', title: 'GitHub', favicon: null },
]

export function hostOf(url: string): string {
  return readUrl(url).host
}

export function essentialFromTab(tab: TabView): Essential {
  return { host: hostOf(tab.url), url: tab.url, title: tab.title, favicon: tab.favicon }
}

export function isEssential(raw: unknown): raw is Essential {
  if (typeof raw !== 'object' || raw === null) return false
  const item = raw as Record<string, unknown> // Justification : garde de type, chaque champ est verifie ci-dessous.
  return (
    typeof item['host'] === 'string' &&
    typeof item['url'] === 'string' &&
    typeof item['title'] === 'string' &&
    (typeof item['favicon'] === 'string' || item['favicon'] === null)
  )
}

/** Premier onglet ouvert sur l'hote de chaque essentiel, et les onglets restants pour la liste. */
export function splitTabs(
  tabs: TabView[],
  essentials: Essential[],
): { represented: Map<string, TabView>; loose: TabView[] } {
  const represented = new Map<string, TabView>()
  const loose: TabView[] = []
  const hosts = new Set(essentials.map((item) => item.host))
  for (const tab of tabs) {
    const host = hostOf(tab.url)
    if (hosts.has(host) && !represented.has(host)) represented.set(host, tab)
    else loose.push(tab)
  }
  return { represented, loose }
}
