// Responsabilite : identite des feuilles qui se posent sur la liste d'onglets.

export type SheetId = 'shield' | 'library' | 'extensions' | 'settings'

export const SHEET_TITLE: Record<SheetId, string> = {
  shield: 'Bouclier',
  library: 'Bibliothèque',
  extensions: 'Extensions',
  settings: 'Réglages',
}
