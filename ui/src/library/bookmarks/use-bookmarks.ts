// Responsabilite : commandes des favoris — ajout depuis l'onglet courant, retrait, reordonnancement.

import { useCallback, useEffect, useMemo, useState } from 'react'
import type { BookmarkView, TabView, UiRequest } from '../../shared/contract'

export interface BookmarksController {
  /** Ordre affiche : celui du coeur, sauf pendant un glisser ou il vit ici. */
  order: BookmarkView[]
  setOrder: (next: BookmarkView[]) => void
  /** Vrai si l'onglet actif est deja dans les favoris. */
  currentSaved: boolean
  canAddCurrent: boolean
  addCurrent: () => void
  remove: (url: string) => void
  /** Envoie la position finale de l'element deplace apres un glisser. */
  commitMove: (url: string) => void
  open: (url: string) => void
}

interface Source {
  bookmarks: BookmarkView[]
  activeTab: TabView | null
}

type Send = (request: UiRequest) => void

function isSavable(tab: TabView | null): tab is TabView {
  // Seules les pages web et les fichiers se mettent en favori : les pages d'Echo ne sont pas des sites.
  return tab !== null && /^(https?|file):/.test(tab.url)
}

/** Ordre local pendant le glisser, realigne sur le coeur des qu'il parle. */
function useLocalOrder(bookmarks: BookmarkView[]): [BookmarkView[], (next: BookmarkView[]) => void] {
  const [order, setOrder] = useState(bookmarks)
  useEffect(() => setOrder(bookmarks), [bookmarks])
  return [order, setOrder]
}

function useCurrentPage(send: Send, bookmarks: BookmarkView[], activeTab: TabView | null): {
  currentSaved: boolean; canAddCurrent: boolean; addCurrent: () => void
} {
  const currentSaved = useMemo(
    () => isSavable(activeTab) && bookmarks.some((item) => item.url === activeTab.url),
    [bookmarks, activeTab],
  )
  const addCurrent = useCallback((): void => {
    if (isSavable(activeTab)) send({ kind: 'addBookmark', id: activeTab.id })
  }, [send, activeTab])
  return { currentSaved, canAddCurrent: isSavable(activeTab) && !currentSaved, addCurrent }
}

export function useBookmarks(send: Send, source: Source): BookmarksController {
  const { bookmarks, activeTab } = source
  const [order, setOrder] = useLocalOrder(bookmarks)
  const current = useCurrentPage(send, bookmarks, activeTab)

  const remove = useCallback((url: string): void => send({ kind: 'removeBookmark', url }), [send])
  const open = useCallback((url: string): void => send({ kind: 'newTab', url }), [send])
  const commitMove = useCallback(
    (url: string): void => {
      const to = order.findIndex((item) => item.url === url)
      const from = bookmarks.findIndex((item) => item.url === url)
      if (to !== -1 && to !== from) send({ kind: 'moveBookmark', url, to })
    },
    [send, order, bookmarks],
  )

  return { order, setOrder, ...current, remove, commitMove, open }
}
