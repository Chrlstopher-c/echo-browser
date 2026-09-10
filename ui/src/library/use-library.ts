// Responsabilite : source de la bibliotheque. Branchement au coeur a venir — voir le contrat.

import { useMemo } from 'react'
import { EMPTY_LIBRARY, type LibraryContent } from './library-model'

export function useLibrary(): LibraryContent {
  // Le contrat ne porte encore aucun evenement bibliotheque : contenu vide, etats vides affiches.
  return useMemo(() => EMPTY_LIBRARY, [])
}
