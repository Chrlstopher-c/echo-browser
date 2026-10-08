// Responsabilite : alerte en bas de la barre quand la machine n'est pas a jour avec son compte Echo, avec la synchro
// a la demande. Reevaluee chaque minute (l'anciennete change sans evenement du coeur).

import { AnimatePresence, motion } from 'framer-motion'
import { useEffect, useState, type ReactElement } from 'react'
import { syncWarning } from '../account/sync-health'
import type { AccountView, UiRequest } from '../shared/contract'
import { IconRefresh, IconWarning } from '../shared/design/icons'
import { QUICK } from '../shared/design/motion'
import { PushButton } from '../shared/design/push-button'

const TICK_MS = 60_000

export interface SyncStripProps {
  account: AccountView | null
  send: (request: UiRequest) => void
}

export function SyncStrip({ account, send }: SyncStripProps): ReactElement {
  const [, setTick] = useState(0)
  useEffect(() => {
    const every = setInterval(() => setTick((n) => n + 1), TICK_MS)
    return () => clearInterval(every)
  }, [])
  const warning = syncWarning(account)
  return (
    <AnimatePresence>
      {warning !== null && (
        <motion.div
          key="synchro"
          role="status"
          aria-label="Synchronisation"
          initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0 }}
          transition={QUICK}
          className="mb-1 flex flex-col gap-2 rounded-row bg-card px-2.5 py-2 text-[11.5px] leading-snug shadow-card"
        >
          <p className="flex items-start gap-1.5 text-ink">
            <IconWarning size={13} className="mt-px shrink-0 text-warn" />
            <span>{warning.message}</span>
          </p>
          <div className="flex gap-1.5">
            <PushButton icon={<IconRefresh size={12} />} onClick={() => send({ kind: 'accountSync' })}>
              {warning.action}
            </PushButton>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}
