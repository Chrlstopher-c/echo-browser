// Responsabilite : modele de la bibliotheque — section affichee (persistee), favoris, historique, telechargements.

import { useCallback, useState } from 'react'
import type { UiRequest } from '../shared/contract'
import type { CoreState } from '../shared/core-state'
import { readLocal, writeLocal } from '../shared/local-store'
import { activeTabOf } from '../shared/core-state'
import { useBookmarks, type BookmarksController } from './bookmarks/use-bookmarks'
import { useDownloads, type DownloadsController } from './downloads/use-downloads'
import { useHistory, type HistoryController } from './history/use-history'

export type LibrarySection = 'bookmarks' | 'history' | 'downloads'

const STORE_KEY = 'echo.library.section'
const SECTIONS: LibrarySection[] = ['bookmarks', 'history', 'downloads']

export interface LibraryController {
  section: LibrarySection
  setSection: (next: LibrarySection) => void
  bookmarks: BookmarksController
  history: HistoryController
  downloads: DownloadsController
}

function isSection(value: unknown): value is LibrarySection {
  return typeof value === 'string' && SECTIONS.some((section) => section === value)
}

function readStored(): LibrarySection {
  return readLocal(STORE_KEY, (raw) => (isSection(raw) ? raw : null)) ?? 'bookmarks'
}

export function useLibrary(send: (request: UiRequest) => void, state: CoreState): LibraryController {
  const [section, setSectionState] = useState<LibrarySection>(readStored)
  const setSection = useCallback((next: LibrarySection): void => {
    setSectionState(next)
    writeLocal(STORE_KEY, next)
  }, [])
  const bookmarks = useBookmarks(send, { bookmarks: state.bookmarks, activeTab: activeTabOf(state) })
  const history = useHistory(send, { entries: state.history, total: state.historyTotal })
  const downloads = useDownloads(send, state.downloads)
  return { section, setSection, bookmarks, history, downloads }
}
