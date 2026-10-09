// Responsabilite : les noms des conteneurs, a portee des lignes d'onglet (sans les faire passer de composant en
// composant).

import { createContext, useContext } from 'react'

export const ContainerNames = createContext<ReadonlyMap<string, string>>(new Map())

export function useContainerName(id: string | null): string | null {
  const names = useContext(ContainerNames)
  return id === null ? null : (names.get(id) ?? 'Conteneur')
}
