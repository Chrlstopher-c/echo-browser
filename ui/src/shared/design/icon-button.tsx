// Responsabilite : bouton icone — unique forme cliquable des rangees de controle de la barre.

import type { ReactElement, ReactNode } from 'react'

export type ButtonTone = 'neutral' | 'guard' | 'warn' | 'danger'

export interface IconButtonProps {
  label: string
  onClick: () => void
  children: ReactNode
  tone?: ButtonTone
  disabled?: boolean
  active?: boolean
  /** Etiquette a droite de l'icone (compteur, par exemple). */
  trailing?: ReactNode
}

const TONE_CLASS: Record<ButtonTone, string> = {
  neutral: 'text-ink-muted hover:text-ink',
  guard: 'text-guard',
  warn: 'text-warn',
  danger: 'text-danger',
}

export function IconButton(props: IconButtonProps): ReactElement {
  const { label, onClick, children, tone = 'neutral', disabled = false, active = false, trailing } = props
  const state = disabled ? 'text-ink-faint/50 cursor-default' : `${TONE_CLASS[tone]} hover:bg-hover`
  const shape = trailing === undefined ? 'size-7 justify-center' : 'h-7 gap-1.5 px-2'
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      aria-pressed={active}
      disabled={disabled}
      onClick={onClick}
      className={`${shape} ${state} flex shrink-0 items-center rounded-row transition-colors duration-100
        ${active ? 'bg-card shadow-card' : ''}`}
    >
      {children}
      {trailing}
    </button>
  )
}
