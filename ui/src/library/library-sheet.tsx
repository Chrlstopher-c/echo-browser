// Responsabilite : feuille de la bibliotheque — trois sections sous un selecteur, le contenu glisse entre elles.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { QUICK } from '../shared/design/motion'
import { Segmented, type Segment } from '../shared/design/segmented'
import { BookmarksPanel } from './bookmarks/bookmarks-panel'
import { DownloadsPanel } from './downloads/downloads-panel'
import { HistoryPanel } from './history/history-panel'
import type { LibraryController, LibrarySection } from './use-library'

function RunningDot(): ReactElement {
  return <span aria-hidden className="size-1.5 rounded-full bg-guard" />
}

function segmentsOf(controller: LibraryController): Array<Segment<LibrarySection>> {
  const running = controller.downloads.summary.running > 0
  return [
    { id: 'bookmarks', label: 'Favoris' },
    { id: 'history', label: 'Historique' },
    { id: 'downloads', label: 'Fichiers', badge: running ? <RunningDot /> : null },
  ]
}

function Panel({ controller }: { controller: LibraryController }): ReactElement {
  switch (controller.section) {
    case 'bookmarks':
      return <BookmarksPanel controller={controller.bookmarks} />
    case 'history':
      return <HistoryPanel controller={controller.history} />
    case 'downloads':
      return <DownloadsPanel controller={controller.downloads} />
  }
}

export function LibrarySheet({ controller }: { controller: LibraryController }): ReactElement {
  return (
    <div className="flex flex-col gap-2">
      <div className="px-2">
        <Segmented name="bibliotheque" segments={segmentsOf(controller)} value={controller.section}
          onChange={controller.setSection} />
      </div>
      <AnimatePresence mode="wait" initial={false}>
        <motion.div
          key={controller.section}
          initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -4 }}
          transition={QUICK}
        >
          <Panel controller={controller} />
        </motion.div>
      </AnimatePresence>
    </div>
  )
}
