// Responsabilite : bandeau de message du coeur, efface automatiquement.

import { AnimatePresence, motion } from 'framer-motion'
import { useEffect, useState } from 'react'
import type { ReactElement } from 'react'
import type { NoticeLevel } from '../shared/contract'
import type { Notice } from '../shared/core-state'

const VISIBLE_MS = 4000

const TONE: Record<NoticeLevel, string> = {
  info: 'text-ink-muted',
  warning: 'text-warn',
  error: 'text-danger',
}

export function NoticeStrip({ notice }: { notice: Notice | null }): ReactElement {
  const [shown, setShown] = useState<Notice | null>(null)

  useEffect(() => {
    setShown(notice)
    if (notice === null) return
    const timer = setTimeout(() => setShown(null), VISIBLE_MS)
    return () => clearTimeout(timer)
  }, [notice])

  return (
    <AnimatePresence>
      {shown !== null && (
        <motion.div
          key={shown.at}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15 }}
          className={`pointer-events-none absolute inset-x-0 top-full z-10 truncate bg-shell/95 px-3 py-1
            text-[11.5px] ${TONE[shown.level]}`}
        >
          {shown.message}
        </motion.div>
      )}
    </AnimatePresence>
  )
}
