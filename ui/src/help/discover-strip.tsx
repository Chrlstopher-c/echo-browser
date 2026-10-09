// Responsabilite : bandeau « a decouvrir », une seule fois par machine : ou trouver l'Aide, la securite du site et
// les profils. Ferme pour de bon au premier clic (memoire locale).

import { AnimatePresence, motion } from 'framer-motion'
import { useState, type ReactElement } from 'react'
import type { UiRequest } from '../shared/contract'
import { IconHelp } from '../shared/design/icons'
import { QUICK } from '../shared/design/motion'
import { PushButton } from '../shared/design/push-button'
import { readLocal, writeLocal } from '../shared/local-store'

const SEEN_KEY = 'echo.decouverte.vue'

/** `ready` : l'accueil du premier lancement est termine (la bulle ne se superpose pas a lui). */
export function DiscoverStrip({ send, ready }: { send: (request: UiRequest) => void; ready: boolean }): ReactElement {
  const [seen, setSeen] = useState(() => readLocal(SEEN_KEY, (raw) => (raw === true ? true : null)) === true)
  const close = (): void => {
    writeLocal(SEEN_KEY, true)
    setSeen(true)
  }
  return (
    <AnimatePresence>
      {!seen && ready && (
        <motion.div key="decouverte" role="status" aria-label="À découvrir" initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} transition={QUICK}
          className="mb-1 flex flex-col gap-2 rounded-row bg-card px-2.5 py-2 text-[11.5px] leading-snug shadow-card">
          <p className="flex items-start gap-1.5 text-ink">
            <IconHelp size={13} className="mt-px shrink-0 text-guard" />
            <span>
              Le cadenas de l’adresse montre avec qui la page communique ; le « + » des pastilles crée un profil.
              Tout le reste est dans l’Aide (F1).
            </span>
          </p>
          <div className="flex gap-1.5">
            <PushButton tone="guard" onClick={() => { send({ kind: 'openPage', page: 'aide' }); close() }}>
              Ouvrir l’Aide
            </PushButton>
            <PushButton onClick={close}>Plus tard</PushButton>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}
