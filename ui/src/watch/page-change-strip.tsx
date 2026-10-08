// Responsabilite : bandeau « cette page a changé » en bas de la barre — pour une page surveillee, les lignes ajoutees
// et retirees depuis la visite precedente. Se ferme d'un clic.

import { AnimatePresence, motion } from 'framer-motion'
import { useState, type ReactElement } from 'react'
import { IconClock } from '../shared/design/icons'
import { QUICK } from '../shared/design/motion'
import { PushButton } from '../shared/design/push-button'
import type { CoreState } from '../shared/core-state'

const SHOWN = 4

function Lines({ lines, sign, tone }: { lines: string[]; sign: string; tone: string }): ReactElement | null {
  if (lines.length === 0) return null
  return (
    <ul className="flex flex-col gap-0.5">
      {lines.slice(0, SHOWN).map((line) => (
        <li key={line} className={`flex gap-1.5 ${tone}`}>
          <span className="numerique shrink-0">{sign}</span>
          <span className="line-clamp-2 [overflow-wrap:anywhere]">{line}</span>
        </li>
      ))}
      {lines.length > SHOWN && <li className="text-ink-faint">et {lines.length - SHOWN} autre(s)</li>}
    </ul>
  )
}

export function PageChangeStrip({ change }: { change: CoreState['pageChange'] }): ReactElement {
  const [seen, setSeen] = useState<CoreState['pageChange']>(null)
  const shown = change !== null && change !== seen
  return (
    <AnimatePresence>
      {shown && (
        <motion.div key={change.url + change.added.join() + change.removed.join()} role="status"
          aria-label="Page modifiée" initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0 }} transition={QUICK}
          className="mb-1 flex flex-col gap-2 rounded-row bg-card px-2.5 py-2 text-[11.5px] leading-snug shadow-card">
          <p className="flex items-start gap-1.5 text-ink">
            <IconClock size={13} className="mt-px shrink-0 text-guard" />
            <span>Cette page a changé depuis votre dernière visite.</span>
          </p>
          <Lines lines={change.added} sign="+" tone="text-guard" />
          <Lines lines={change.removed} sign="−" tone="text-danger" />
          <div><PushButton onClick={() => setSeen(change)}>Vu</PushButton></div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}
