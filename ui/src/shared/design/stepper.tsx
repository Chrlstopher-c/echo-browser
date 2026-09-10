// Responsabilite : reglage numerique — moins, valeur, plus. La valeur est bornee et affichee avec son unite.

import type { ReactElement } from 'react'
import { IconMinus, IconPlus } from './icons'

export interface StepperProps {
  value: number
  min: number
  max: number
  step: number
  unit: string
  label: string
  onChange: (next: number) => void
  /** Rend la valeur ; par defaut, le nombre suivi de l'unite. */
  render?: (value: number) => string
}

function StepButton({ label, disabled, onClick, children }: {
  label: string; disabled: boolean; onClick: () => void; children: ReactElement
}): ReactElement {
  return (
    <button
      type="button"
      aria-label={label}
      disabled={disabled}
      onClick={onClick}
      className="grid size-6 place-items-center rounded-md text-ink-muted transition-colors duration-100
        hover:bg-ink/10 hover:text-ink disabled:text-ink-faint/40 disabled:hover:bg-transparent"
    >
      {children}
    </button>
  )
}

export function Stepper(props: StepperProps): ReactElement {
  const { value, min, max, step, unit, label, onChange, render } = props
  const clamp = (next: number): number => Math.min(max, Math.max(min, Math.round(next / step) * step))
  const text = render !== undefined ? render(value) : `${value}${unit.length > 0 ? ` ${unit}` : ''}`
  return (
    <div className="flex h-7 shrink-0 items-center gap-0.5 rounded-row bg-field px-0.5 shadow-card">
      <StepButton label={`${label} : moins`} disabled={value <= min} onClick={() => onChange(clamp(value - step))}>
        <IconMinus size={12} />
      </StepButton>
      <span className="numerique min-w-[5ch] text-center text-[11.5px] text-ink">{text}</span>
      <StepButton label={`${label} : plus`} disabled={value >= max} onClick={() => onChange(clamp(value + step))}>
        <IconPlus size={12} />
      </StepButton>
    </div>
  )
}
