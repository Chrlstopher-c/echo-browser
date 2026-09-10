// Responsabilite : interrupteur binaire du systeme de design. Vert « garde » a l'etat actif.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { QUICK } from './motion'

export interface ToggleProps {
  checked: boolean
  onChange: (next: boolean) => void
  label: string
  disabled?: boolean
}

export function Toggle({ checked, onChange, label, disabled = false }: ToggleProps): ReactElement {
  const track = checked ? 'bg-guard' : 'bg-hairline'
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative h-[18px] w-8 shrink-0 rounded-full transition-colors duration-150
        ${disabled ? 'bg-hairline/40 cursor-default' : track}`}
    >
      <motion.span
        animate={{ x: checked ? 14 : 0 }}
        transition={QUICK}
        className={`absolute top-[2px] left-[2px] size-[14px] rounded-full bg-ink shadow-sm
          ${disabled ? 'opacity-50' : ''}`}
      />
    </button>
  )
}
