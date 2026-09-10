// Responsabilite : conteneur anime des panneaux. Un seul panneau visible, hauteur donnee par `panel.ts`.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement, ReactNode } from 'react'
import { PANEL_HEIGHT, type PanelId } from './panel'

export interface PanelHostProps {
  panel: PanelId | null
  children: ReactNode
}

export function PanelHost({ panel, children }: PanelHostProps): ReactElement {
  return (
    <AnimatePresence initial={false}>
      {panel !== null && (
        <motion.div
          key={panel}
          initial={{ opacity: 0, y: -6 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -6 }}
          transition={{ duration: 0.14, ease: [0.22, 0.61, 0.36, 1] }}
          style={{ height: PANEL_HEIGHT[panel] }}
          className="shrink-0 overflow-hidden border-b border-hairline bg-surface"
        >
          {children}
        </motion.div>
      )}
    </AnimatePresence>
  )
}
