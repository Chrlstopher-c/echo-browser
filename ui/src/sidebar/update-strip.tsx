// Responsabilite : annoncer qu'une nouvelle version d'Echo est prete, et proposer le redemarrage qui l'applique.

import { AnimatePresence, motion } from 'framer-motion'
import { useState, type ReactElement } from 'react'
import type { UiRequest, UpdateView } from '../shared/contract'
import { QUICK } from '../shared/design/motion'
import { PushButton } from '../shared/design/push-button'

export interface UpdateStripProps {
  update: UpdateView | null
  send: (request: UiRequest) => void
}

export function UpdateStrip({ update, send }: UpdateStripProps): ReactElement {
  const [later, setLater] = useState<string | null>(null)
  const ready = update !== null && update.status === 'ready' && update.latest !== null && later !== update.latest
  return (
    <AnimatePresence>
      {ready && (
        <motion.div
          key="mise-a-jour"
          role="status"
          aria-label="Mise à jour"
          initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0 }}
          transition={QUICK}
          className="mb-1 flex flex-col gap-2 rounded-row bg-card px-2.5 py-2 text-[11.5px] leading-snug shadow-card"
        >
          <p className="text-ink">
            Echo <span className="font-medium">{update.latest}</span> est prêt. Redémarrer pour l’installer ? Vos
            onglets sont gardés.
          </p>
          <div className="flex gap-1.5">
            <PushButton tone="guard" onClick={() => send({ kind: 'restartBrowser' })}>Redémarrer</PushButton>
            <PushButton onClick={() => setLater(update.latest)}>Plus tard</PushButton>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}
