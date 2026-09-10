// Responsabilite : cadre anime d'une feuille — en-tete avec titre et fermeture, corps defilant.
// La feuille se pose sur la liste d'onglets, dans la meme colonne : rien ne s'ouvre a cote.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement, ReactNode } from 'react'
import { IconButton } from '../shared/design/icon-button'
import { IconClose } from '../shared/design/icons'
import { PANEL } from '../shared/design/motion'
import { SHEET_TITLE, type SheetId } from './sheet'
import type { SheetController } from './use-sheet'

export interface SheetHostProps {
  sheet: SheetController
  children: ReactNode
}

function SheetHeader({ id, onClose }: { id: SheetId; onClose: () => void }): ReactElement {
  return (
    <header className="flex h-8 shrink-0 items-center justify-between pr-0.5 pl-2">
      <h2 className="intitule truncate">{SHEET_TITLE[id]}</h2>
      <IconButton label="Fermer (Échap)" onClick={onClose}>
        <IconClose size={13} />
      </IconButton>
    </header>
  )
}

export function SheetHost({ sheet, children }: SheetHostProps): ReactElement {
  return (
    <AnimatePresence initial={false}>
      {sheet.current !== null && (
        <motion.section
          key={sheet.current}
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 6 }}
          transition={PANEL}
          className="absolute inset-0 z-10 flex flex-col"
        >
          <SheetHeader id={sheet.current} onClose={sheet.close} />
          <div className="min-h-0 flex-1 overflow-y-auto pt-1 pb-2">{children}</div>
        </motion.section>
      )}
    </AnimatePresence>
  )
}
