// Responsabilite : confirmation en place d'une action destructrice — pas de boite modale, la ligne
// se retourne pour poser la question, et se retourne a nouveau sur la reponse.

import type { ReactElement, ReactNode } from 'react'
import { PushButton } from './push-button'

export interface ConfirmStripProps {
  question: ReactNode
  confirmLabel: string
  onConfirm: () => void
  onCancel: () => void
}

export function ConfirmStrip({ question, confirmLabel, onConfirm, onCancel }: ConfirmStripProps): ReactElement {
  return (
    <div className="flex items-center gap-2 rounded-row bg-hover px-2.5 py-2">
      <p className="min-w-0 flex-1 text-[11.5px] leading-snug text-ink-muted">{question}</p>
      <button
        type="button"
        onClick={onCancel}
        className="shrink-0 rounded-md px-2 py-[3px] text-[11px] text-ink-muted hover:text-ink"
      >
        Annuler
      </button>
      <PushButton tone="danger" onClick={onConfirm}>
        {confirmLabel}
      </PushButton>
    </div>
  )
}
