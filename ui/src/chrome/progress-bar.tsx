// Responsabilite : barre de progression fine sous la barre de navigation.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'

export interface ProgressBarProps {
  loading: boolean
  progress: number
}

export function ProgressBar({ loading, progress }: ProgressBarProps): ReactElement {
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
            transition={{ duration: 0.22, ease: [0.22, 0.61, 0.36, 1] }}
          />
        )}
      </AnimatePresence>
    </div>
  )
}
