// Responsabilite : proposition de routine en bas de la barre — une suite de sites revient souvent ; la creer (nommee)
// ou ne plus la proposer. Disparait des qu'une reponse est donnee.

import { AnimatePresence, motion } from 'framer-motion'
import { useEffect, useState, type ReactElement } from 'react'
import type { RoutineProposalView, UiRequest } from '../shared/contract'
import { IconSparkle } from '../shared/design/icons'
import { QUICK } from '../shared/design/motion'
import { PushButton } from '../shared/design/push-button'

export interface RoutineStripProps {
  proposal: RoutineProposalView | null
  send: (request: UiRequest) => void
}

export function RoutineStrip({ proposal, send }: RoutineStripProps): ReactElement {
  const [answered, setAnswered] = useState<string | null>(null)
  const [name, setName] = useState('')
  useEffect(() => setName(''), [proposal?.fingerprint])
  const shown = proposal !== null && answered !== proposal.fingerprint
  const answer = (accept: boolean): void => {
    if (proposal === null) return
    setAnswered(proposal.fingerprint)
    send(accept ? { kind: 'routineAccept', fingerprint: proposal.fingerprint, name }
      : { kind: 'routineDismiss', fingerprint: proposal.fingerprint })
  }
  return (
    <AnimatePresence>
      {shown && (
        <motion.div key={proposal.fingerprint} role="status" aria-label="Routine" initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} transition={QUICK}
          className="mb-1 flex flex-col gap-2 rounded-row bg-card px-2.5 py-2 text-[11.5px] leading-snug shadow-card">
          <p className="flex items-start gap-1.5 text-ink">
            <IconSparkle size={13} className="mt-px shrink-0 text-guard" />
            <span>Vous ouvrez souvent <span className="font-medium">{proposal.sites.join(' → ')}</span>. En faire
              une routine ?</span>
          </p>
          <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Nom (ex. Matin)"
            aria-label="Nom de la routine"
            className="h-7 rounded-row bg-field px-2 text-[11.5px] text-ink shadow-field outline-none
              placeholder:text-ink-faint" />
          <div className="flex gap-1.5">
            <PushButton tone="guard" onClick={() => answer(true)}>Créer</PushButton>
            <PushButton onClick={() => answer(false)}>Non merci</PushButton>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}
