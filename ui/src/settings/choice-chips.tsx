// Responsabilite : choix parmi une liste fermee (moteur de recherche…) — pastilles en relief, la choisie enfoncee.

import type { ReactElement } from 'react'
import type { ChoiceOption } from './setting-catalogue'

export interface ChoiceChipsProps {
  options: ChoiceOption[]
  value: string
  label: string
  onChange: (next: string) => void
}

export function ChoiceChips({ options, value, label, onChange }: ChoiceChipsProps): ReactElement {
  // Une ancienne valeur libre (« google », une adresse) retombe sur le premier choix, comme le fait le coeur.
  const current = options.some((option) => option.id === value) ? value : options[0]?.id
  return (
    <div role="radiogroup" aria-label={label} className="flex flex-wrap gap-1.5">
      {options.map((option) => {
        const active = option.id === current
        return (
          <button key={option.id} type="button" role="radio" aria-checked={active}
            onClick={() => onChange(option.id)}
            className={`rounded-full px-2.5 py-1 text-[11.5px] transition-shadow duration-100
              ${active ? 'bg-field text-ink shadow-field ring-1 ring-guard/60' : 'bg-card text-ink-muted shadow-card hover:text-ink'}`}>
            {option.label}
          </button>
        )
      })}
    </div>
  )
}
