// Responsabilite : identite des feuilles qui se posent sur la liste d'onglets.

export type SheetId = 'shield' | 'extensions' | 'downloads' | 'settings' | 'bookmarks' | 'history'

export const SHEET_TITLE: Record<SheetId, string> = {
  shield: 'Bouclier',
  extensions: 'Extensions',
  downloads: 'Téléchargements',
  settings: 'Réglages',
  bookmarks: 'Favoris',
  history: 'Historique',
}
