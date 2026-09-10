// Responsabilite : trait de progression au bas du champ d'adresse. Il suit l'avancement reel ;
// a la fin, il court jusqu'au bout puis s'efface — l'oeil voit la page arriver, jamais un saut.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { CHROME_EASE, TRACK } from '../shared/design/motion'

export interface LoadProgressProps {
  loading: boolean
  progress: number
}

export function LoadProgress({ loading, progress }: LoadProgressProps): ReactElement {
  const ratio = Math.min(Math.max(progress, 0), 1)
  return (
    <div className="pointer-events-none absolute inset-x-0 bottom-0 h-[2px] overflow-hidden">
      <AnimatePresence>
        {loading && (
          <motion.div
            key="progression"
            className="h-full origin-left rounded-r-full bg-guard"
            initial={{ scaleX: 0, opacity: 1 }}
            animate={{ scaleX: Math.max(ratio, 0.03), opacity: 1 }}
            exit={{ scaleX: 1, opacity: 0, transition: { scaleX: TRACK, opacity: { duration: 0.28, delay: 0.16 } } }}
            transition={TRACK}
          />
        )}
      </AnimatePresence>
      <AnimatePresence>
        {loading && (
          <motion.div
            key="lueur"
            aria-hidden
            initial={{ x: '-40%', opacity: 0 }}
            animate={{ x: '260%', opacity: [0, 0.6, 0] }}
            exit={{ opacity: 0 }}
            transition={{ duration: 1.6, ease: CHROME_EASE, repeat: Infinity, repeatDelay: 0.4 }}
            className="absolute inset-y-0 left-0 w-1/3 bg-gradient-to-r from-transparent via-ink/40 to-transparent"
          />
        )}
      </AnimatePresence>
    </div>
  )
}
