// Responsabilite : la question d'un site qui demande une permission (camera, micro, position…).
// Posee au-dessus de la rangee d'outils tant qu'elle attend une reponse.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { UiRequest } from '../shared/contract'
import type { PermissionRequest } from '../shared/core-state'
import { QUICK } from '../shared/design/motion'
import { PushButton } from '../shared/design/push-button'
import { readUrl } from '../shared/url-shape'

const LABELS: Record<string, string> = {
  camera: 'la caméra',
  microphone: 'le micro',
  position: 'votre position',
  notifications: 'les notifications',
  'presse-papiers': 'le presse-papiers',
}

function describe(kinds: string[]): string {
  const names = kinds.map((kind) => LABELS[kind] ?? kind)
  return names.length > 1 ? `${names.slice(0, -1).join(', ')} et ${names[names.length - 1]}` : (names[0] ?? '')
}

export interface PermissionStripProps {
  requests: PermissionRequest[]
  send: (request: UiRequest) => void
}

export function PermissionStrip({ requests, send }: PermissionStripProps): ReactElement {
  const current = requests[0]
  return (
    <AnimatePresence>
      {current !== undefined && (
        <motion.div
          key={current.id}
          role="alertdialog"
          initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0 }}
          transition={QUICK}
          className="mb-1 flex flex-col gap-2 rounded-row bg-card px-2.5 py-2 text-[11.5px] shadow-card"
        >
          <p className="text-ink">
            <span className="font-medium">{readUrl(current.origin).host || current.origin}</span>
            {' '}veut utiliser {describe(current.kinds)}.
          </p>
          <div className="flex gap-1.5">
            <PushButton tone="guard" onClick={() => send({ kind: 'answerPermission', id: current.id, allow: true, remember: true })}>
              Autoriser
            </PushButton>
            <PushButton onClick={() => send({ kind: 'answerPermission', id: current.id, allow: false, remember: true })}>
              Refuser
            </PushButton>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}
