// Responsabilite : cadre anime d'une feuille — en-tete avec retour et fermeture, corps defilant.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement, ReactNode } from 'react'
import { IconButton } from '../shared/design/icon-button'
import { IconChevronLeft, IconClose } from '../shared/design/icons'
import { SHEET_TITLE, type SheetId } from './sheet'
import { CHROME_EASE } from './sidebar-geometry'
import type { SheetController } from './use-sheet'

export interface SheetHostProps {
  sheet: SheetController
  children: ReactNode
  footer?: ReactNode
}

function SheetHeader({ sheet, id }: { sheet: SheetController; id: SheetId }): ReactElement {
  return (
    <header className="flex h-8 shrink-0 items-center gap-1 pr-0.5">
      {sheet.canGoBack && (
        <IconButton label="Retour" onClick={sheet.back}>
          <IconChevronLeft size={14} />
        </IconButton>
      )}
      <h2 className={`flex-1 truncate text-[11px] font-medium tracking-[0.08em] text-ink-muted uppercase
        ${sheet.canGoBack ? '' : 'pl-2'}`}>
        {SHEET_TITLE[id]}
      </h2>
      <IconButton label="Fermer" onClick={sheet.close}>
        <IconClose size={13} />
      </IconButton>
    </header>
  )
}

export function SheetHost({ sheet, children, footer }: SheetHostProps): ReactElement {
  return (
    <AnimatePresence initial={false}>
      {sheet.current !== null && (
        <motion.section
          key={sheet.current}
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 6 }}
          transition={{ duration: 0.18, ease: CHROME_EASE }}
          className="absolute inset-0 z-10 flex flex-col"
        >
          <SheetHeader sheet={sheet} id={sheet.current} />
          <div className="min-h-0 flex-1 overflow-y-auto pb-2">{children}</div>
          {footer !== undefined && (
            <footer className="shrink-0 border-t border-hairline px-2 pt-2 text-[11px] text-ink-faint">{footer}</footer>
          )}
        </motion.section>
      )}
    </AnimatePresence>
  )
}
