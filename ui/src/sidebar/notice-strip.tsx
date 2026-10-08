// Responsabilite : message du coeur, pose au-dessus de la rangee d'outils, efface automatiquement ; ses boutons
// (« Ouvrir », « Retirer »…) renvoient leur requete au coeur. Le survol suspend l'effacement.

import { AnimatePresence, motion } from 'framer-motion'
import { useEffect, useState } from 'react'
import type { ReactElement } from 'react'
import type { NoticeLevel, UiRequest } from '../shared/contract'
import type { Notice } from '../shared/core-state'
import { QUICK } from '../shared/design/motion'

const VISIBLE_MS = 4000
const WITH_ACTIONS_MS = 8000

const TONE: Record<NoticeLevel, string> = {
  info: 'text-ink-muted',
  warning: 'text-warn',
  error: 'text-danger',
}

export function NoticeStrip({ notice, send }: { notice: Notice | null; send: (request: UiRequest) => void }):
  ReactElement {
  const [shown, setShown] = useState<Notice | null>(null)
  const [hover, setHover] = useState(false)

  useEffect(() => {
    setShown(notice)
  }, [notice])
  useEffect(() => {
    if (shown === null || hover) return
    const timer = setTimeout(() => setShown(null), shown.actions.length > 0 ? WITH_ACTIONS_MS : VISIBLE_MS)
    return () => clearTimeout(timer)
  }, [shown, hover])

  return (
    <AnimatePresence>
      {shown !== null && (
        <motion.div
          key={shown.at}
          role="status"
          onMouseEnter={() => setHover(true)}
          onMouseLeave={() => setHover(false)}
          initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0 }}
          transition={QUICK}
          className="mb-1 flex flex-col gap-1 rounded-row bg-card px-2.5 py-1.5 shadow-card"
        >
          <p className={`line-clamp-2 text-[11.5px] ${TONE[shown.level]}`}>{shown.message}</p>
          {shown.actions.length > 0 && (
            <div className="flex flex-wrap gap-1">
              {shown.actions.map((action) => (
                <button key={action.label} type="button"
                  onClick={() => {
                    send(action.request)
                    setShown(null)
                  }}
                  className="rounded-full bg-field px-2 py-0.5 text-[11px] text-ink shadow-field hover:text-guard">
                  {action.label}
                </button>
              ))}
            </div>
          )}
        </motion.div>
      )}
    </AnimatePresence>
  )
}
