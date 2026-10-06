// Responsabilite : savoir sur quel dossier un onglet a ete lache, d'apres le point de lacher.

import type { DropPoint } from './tab-row'

/** Le dossier sous le point, `null` pour la liste libre, `undefined` si on a lache hors de la zone des onglets. */
export function folderAt(point: DropPoint): string | null | undefined {
  for (const element of document.elementsFromPoint(point.x, point.y)) {
    if (!(element instanceof HTMLElement)) continue
    const folder = element.closest<HTMLElement>('[data-folder-id]')
    if (folder !== null) return folder.dataset.folderId ?? null
    if (element.closest('[data-loose-tabs]') !== null) return null
  }
  return undefined
}
