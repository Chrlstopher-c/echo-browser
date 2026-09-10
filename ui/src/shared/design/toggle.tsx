// Responsabilite : interrupteur binaire du systeme de design. Vert « garde » a l'etat actif.

import type { ReactElement } from 'react'

export interface ToggleProps {
  checked: boolean
  onChange: (next: boolean) => void
  label: string
  disabled?: boolean
}

export function Toggle({ checked, onChange, label, disabled = false }: ToggleProps): ReactElement {
  const track = checked ? 'bg-guard' : 'bg-edge'
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative h-[18px] w-8 shrink-0 rounded-full transition-colors duration-150
        ${disabled ? 'bg-edge/40 cursor-default' : track}`}
    >
      <span
        className={`absolute top-[2px] size-[14px] rounded-full bg-ink shadow-sm transition-[left] duration-150
          ${checked ? 'left-[16px]' : 'left-[2px]'} ${disabled ? 'opacity-50' : ''}`}
      />
    </button>
  )
}
