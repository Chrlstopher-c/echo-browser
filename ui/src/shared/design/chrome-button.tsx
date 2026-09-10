// Responsabilite : bouton icone de la bande de chrome — unique forme cliquable de la barre d'outils.

import type { ReactElement, ReactNode } from 'react'

export type ButtonTone = 'neutral' | 'guard' | 'warn' | 'danger'

export interface ChromeButtonProps {
  label: string
  onClick: () => void
  children: ReactNode
  tone?: ButtonTone
  disabled?: boolean
  active?: boolean
  compact?: boolean
}

const TONE_CLASS: Record<ButtonTone, string> = {
  neutral: 'text-ink-muted hover:text-ink',
  guard: 'text-guard hover:text-guard',
  warn: 'text-warn hover:text-warn',
  danger: 'text-danger hover:text-danger',
}

export function ChromeButton({
  label,
  onClick,
  children,
  tone = 'neutral',
  disabled = false,
  active = false,
  compact = false,
}: ChromeButtonProps): ReactElement {
  const size = compact ? 'size-6' : 'size-7'
  const state = disabled
    ? 'text-ink-faint/50 cursor-default'
    : `${TONE_CLASS[tone]} hover:bg-hover active:bg-raised`
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      aria-pressed={active}
      disabled={disabled}
      onClick={onClick}
      className={`${size} ${state} grid shrink-0 place-items-center rounded-md transition-colors duration-100
        ${active ? 'bg-raised' : ''}`}
    >
      {children}
    </button>
  )
}
