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
  // Piste en creux, bouton en relief : la matiere reste la meme, seul le bouton prend la couleur « garde ».
  const knob = checked ? 'bg-guard' : 'bg-card'
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative h-[22px] w-10 shrink-0 rounded-full bg-field shadow-field transition-shadow duration-150
        ${disabled ? 'opacity-50 cursor-default' : ''}`}
    >
      <motion.span
        animate={{ x: checked ? 18 : 0 }}
        transition={QUICK}
        className={`absolute top-[3px] left-[3px] size-[16px] rounded-full shadow-card ${knob}`}
      />
    </button>
  )
}
