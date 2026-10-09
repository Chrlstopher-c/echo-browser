// Responsabilite : ce dont les lignes d'onglet ont besoin sans le recevoir de composant en composant : les noms des
// conteneurs et l'envoi d'une demande au coeur.

import { createContext, useContext } from 'react'
import type { UiRequest } from '../shared/contract'

export const ContainerNames = createContext<ReadonlyMap<string, string>>(new Map())

export function useContainerName(id: string | null): string | null {
  const names = useContext(ContainerNames)
  return id === null ? null : (names.get(id) ?? 'Conteneur')
}

export const TabSend = createContext<(request: UiRequest) => void>(() => undefined)
