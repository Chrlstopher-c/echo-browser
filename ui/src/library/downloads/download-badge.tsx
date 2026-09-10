// Responsabilite : badge des telechargements en cours sur le bouton bibliotheque — un anneau qui avance.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { QUICK } from '../../shared/design/motion'
import { ProgressRing } from '../../shared/design/progress-ring'
import type { DownloadsSummary } from './download-reading'

export function DownloadBadge({ summary }: { summary: DownloadsSummary }): ReactElement {
  return (
    <AnimatePresence>
      {summary.running > 0 && (
        <motion.span
          key="badge"
          initial={{ opacity: 0, scale: 0.6 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.6 }}
          transition={QUICK}
          title={`${summary.running} téléchargement${summary.running > 1 ? 's' : ''} en cours`}
          className="pointer-events-none absolute -top-0.5 -right-0.5 grid size-3.5 place-items-center rounded-full
            bg-shell text-guard"
        >
          {summary.progress === null ? (
            <motion.span animate={{ rotate: 360 }} transition={{ duration: 1.2, ease: 'linear', repeat: Infinity }}
              className="grid place-items-center">
              <ProgressRing progress={0.3} size={10} stroke={1.5} />
            </motion.span>
          ) : (
            <ProgressRing progress={summary.progress} size={10} stroke={1.5} />
          )}
        </motion.span>
      )}
    </AnimatePresence>
  )
}
