// Responsabilite : trait de progression au bas du champ d'adresse.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { CHROME_EASE } from '../sidebar/sidebar-geometry'

export interface LoadProgressProps {
  loading: boolean
  progress: number
}

export function LoadProgress({ loading, progress }: LoadProgressProps): ReactElement {
  const ratio = Math.min(Math.max(progress, 0), 1)
  return (
    <div className="pointer-events-none absolute inset-x-0 bottom-0 h-[2px]">
      <AnimatePresence>
        {loading && (
          <motion.div
            key="progression"
            className="h-full origin-left bg-guard"
            initial={{ scaleX: 0, opacity: 1 }}
            animate={{ scaleX: Math.max(ratio, 0.04), opacity: 1 }}
            exit={{ scaleX: 1, opacity: 0 }}
            transition={{ duration: 0.22, ease: CHROME_EASE }}
          />
        )}
      </AnimatePresence>
    </div>
  )
}
