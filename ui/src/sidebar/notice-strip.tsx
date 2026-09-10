// Responsabilite : message du coeur, pose au-dessus de la rangee d'outils, efface automatiquement.

import { AnimatePresence, motion } from 'framer-motion'
import { useEffect, useState } from 'react'
import type { ReactElement } from 'react'
import type { NoticeLevel } from '../shared/contract'
import type { Notice } from '../shared/core-state'
import { QUICK } from '../shared/design/motion'

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
        <motion.p
          key={shown.at}
          role="status"
          initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0 }}
          transition={QUICK}
          className={`mb-1 truncate rounded-row bg-card px-2.5 py-1.5 text-[11.5px] shadow-card ${TONE[shown.level]}`}
        >
          {shown.message}
        </motion.p>
      )}
    </AnimatePresence>
  )
}
