// Responsabilite : bande de relance — propose le redemarrage sans l'imposer, jamais bloquante.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { RESTART_HINT, RESTART_NOTICE } from '../extensions/extension-model'
import { IconReload } from '../shared/design/icons'
import { QUICK } from '../shared/design/motion'
import { PushButton } from '../shared/design/push-button'

export interface RestartStripProps {
  pending: boolean
  /** Nombre d'extensions en attente ; 0 affiche l'avertissement generique. */
  count: number
  onRestart: () => void
}

function label(count: number): string {
  if (count === 0) return RESTART_NOTICE
  return count === 1 ? '1 extension attend la relance.' : `${count} extensions attendent la relance.`
}

export function RestartStrip({ pending, count, onRestart }: RestartStripProps): ReactElement {
  return (
    <AnimatePresence initial={false}>
      {pending && (
        <motion.div
          initial={{ opacity: 0, y: 6, height: 0 }}
          animate={{ opacity: 1, y: 0, height: 'auto' }}
          exit={{ opacity: 0, y: 4, height: 0 }}
          transition={QUICK}
          className="overflow-hidden"
        >
          <div className="mb-1.5 flex items-center gap-2 rounded-row border border-warn/25 bg-warn/10 px-2.5 py-2">
            <div className="min-w-0 flex-1">
              <p className="truncate text-[11.5px] text-warn">{label(count)}</p>
              <p className="truncate text-[10.5px] text-ink-faint">{RESTART_HINT}</p>
            </div>
            <PushButton tone="warn" onClick={onRestart} icon={<IconReload size={12} />}>
              Relancer
            </PushButton>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}
