// Responsabilite : les fiches proposees quand l'utilisateur entre dans un champ reconnu — sous l'adresse, en haut de
// la barre, au plus pres de la page (et non en bas). Un clic remplit ; la proposition s'efface seule.

import { AnimatePresence, motion } from 'framer-motion'
import { useEffect, useState, type ReactElement } from 'react'
import type { NoticeAction, UiRequest } from '../shared/contract'
import { IconUser } from '../shared/design/icons'
import { QUICK } from '../shared/design/motion'

const VISIBLE_MS = 12_000

export function FormOffer(props: { offer: { actions: NoticeAction[]; at: number } | null;
  send: (request: UiRequest) => void }): ReactElement {
  const { offer, send } = props
  const [shown, setShown] = useState(offer)
  useEffect(() => {
    setShown(offer)
    if (offer === null) return
    const timer = setTimeout(() => setShown(null), VISIBLE_MS)
    return () => clearTimeout(timer)
  }, [offer])
  return (
    <AnimatePresence>
      {shown !== null && (
        <motion.div key={shown.at} role="status" aria-label="Remplir le formulaire"
          initial={{ opacity: 0, y: -4 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} transition={QUICK}
          className="flex flex-wrap items-center gap-1.5 rounded-row bg-card px-2.5 py-1.5 shadow-card">
          <IconUser size={12} className="shrink-0 text-guard" />
          <span className="text-[11.5px] text-ink-muted">Remplir ce formulaire avec :</span>
          {shown.actions.map((action) => (
            <button key={action.label} type="button"
              onClick={() => { send(action.request); setShown(null) }}
              className="rounded-full bg-field px-2 py-0.5 text-[11.5px] text-ink shadow-field hover:text-guard">
              {action.label}
            </button>
          ))}
        </motion.div>
      )}
    </AnimatePresence>
  )
}
