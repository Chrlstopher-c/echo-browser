// Responsabilite : bouton texte compact — actions nommees des feuilles (rafraichir, relancer, effacer).

import type { ReactElement, ReactNode } from 'react'
import type { ButtonTone } from './icon-button'

export interface PushButtonProps {
  children: ReactNode
  onClick: () => void
  tone?: ButtonTone
  disabled?: boolean
  /** Icone a gauche du libelle. */
  icon?: ReactNode
}

const TONE_CLASS: Record<ButtonTone, string> = {
  neutral: 'bg-card text-ink-muted shadow-card hover:text-ink',
  guard: 'bg-guard/15 text-guard hover:bg-guard/25',
  warn: 'bg-warn/20 text-warn hover:bg-warn/30',
  danger: 'bg-danger/15 text-danger hover:bg-danger/25',
}

export function PushButton(props: PushButtonProps): ReactElement {
  const { children, onClick, tone = 'neutral', disabled = false, icon } = props
  const state = disabled ? 'bg-hairline/40 text-ink-faint cursor-default' : TONE_CLASS[tone]
  return (
    <button
      type="button"
      disabled={disabled}
      onClick={onClick}
      className={`flex h-6 shrink-0 items-center gap-1.5 rounded-md px-2 text-[11px] font-medium
        transition-colors duration-100 ${state}`}
    >
      {icon}
      {children}
    </button>
  )
}
