// Responsabilite : modele des panneaux bibliotheque. Le coeur ne les alimente pas encore :
// la source reste vide et l'interface affiche ses etats vides.

export type LibrarySection = 'bookmarks' | 'history' | 'downloads'

export interface Bookmark {
  id: string
  title: string
  url: string
}

export interface HistoryEntry {
  id: string
  title: string
  url: string
  /** Horodatage de la visite, en millisecondes. */
  visitedAt: number
}

export type DownloadState = 'running' | 'done' | 'failed'

export interface DownloadItem {
  id: string
  filename: string
  state: DownloadState
  /** Part telechargee, de 0 a 1. */
  progress: number
}

export interface LibraryContent {
  bookmarks: Bookmark[]
  history: HistoryEntry[]
  downloads: DownloadItem[]
}

export const EMPTY_LIBRARY: LibraryContent = { bookmarks: [], history: [], downloads: [] }

/** Heure courte d'une visite, pour la colonne de droite de l'historique. */
export function shortTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' })
}
