// Responsabilite : les conteneurs d'onglets. Un conteneur = des cookies et des comptes a part : deux onglets
// du meme site, dans deux conteneurs, sont deux sessions distinctes. La liste (nom) vit dans le reglage
// `tabs.containers` ; le coeur ne connait que l'identifiant, qui nomme le dossier de donnees.

import { useCallback, useMemo } from 'react'
import type { SettingView, TabId, UiRequest } from '../shared/contract'

export interface Container {
  id: string
  name: string
}

export interface ContainerActions {
  containers: Container[]
  /** Cree un conteneur ; avec `tabId`, y rouvre aussitot cet onglet. */
  create: (tabId?: TabId) => void
  rename: (id: string, name: string) => void
  /** Retire le conteneur de la liste ; ses donnees restent sur le disque. */
  remove: (id: string) => void
  openTab: (id: string) => void
  moveTab: (tabId: TabId, container: string | null) => void
}

const KEY = 'tabs.containers'
const HUES = [8, 32, 140, 205, 275]

/** Une couleur stable pour un conteneur, deduite de son identifiant. */
export function containerColor(id: string): string {
  let sum = 0
  for (const char of id) sum += char.charCodeAt(0)
  return `hsl(${HUES[sum % HUES.length] ?? 205} 58% 62%)`
}

/**
 * Le conteneur choisi par l'utilisateur, a partir du contexte de l'onglet : le coeur prefixe un conteneur par son
 * profil (`profil-<id>--<conteneur>`). Null pour le contexte propre au profil ou le contexte commun.
 */
export function chosenContainer(context: string | null): string | null {
  if (context === null) return null
  const parts = context.split('--')
  if (parts.length > 1) return parts[parts.length - 1] ?? null
  return context.startsWith('profil-') ? null : context
}

function isContainer(value: unknown): value is Container {
  if (typeof value !== 'object' || value === null) return false
  const candidate = value as Record<string, unknown>
  return typeof candidate.id === 'string' && typeof candidate.name === 'string'
}

export function parseContainers(settings: SettingView[]): Container[] {
  const raw = settings.find((item) => item.key === KEY)?.value
  if (raw === undefined || raw.type !== 'text') return []
  try {
    const parsed: unknown = JSON.parse(raw.value)
    return Array.isArray(parsed) ? parsed.filter(isContainer).map(({ id, name }) => ({ id, name })) : []
  } catch (error) {
    console.warn('conteneurs illisibles', error)
    return []
  }
}

export function useContainers(send: (request: UiRequest) => void, settings: SettingView[]): ContainerActions {
  const containers = useMemo(() => parseContainers(settings), [settings])
  const save = useCallback(
    (next: Container[]): void =>
      send({ kind: 'updateSetting', key: KEY, value: { type: 'text', value: JSON.stringify(next) } }),
    [send],
  )
  const moveTab = useCallback(
    (id: TabId, container: string | null): void => send({ kind: 'setTabContainer', id, container }),
    [send],
  )
  return useMemo(
    () => ({
      containers,
      create: (tabId) => {
        const id = `c${Date.now().toString(36)}`
        save([...containers, { id, name: `Compte ${containers.length + 1}` }])
        if (tabId === undefined) send({ kind: 'newTab', container: id })
        else moveTab(tabId, id)
      },
      rename: (id, name) =>
        save(containers.map((item) => (item.id === id ? { ...item, name: name.trim() || item.name } : item))),
      remove: (id) => save(containers.filter((item) => item.id !== id)),
      openTab: (id) => send({ kind: 'newTab', container: id }),
      moveTab,
    }),
    [containers, save, moveTab, send],
  )
}
