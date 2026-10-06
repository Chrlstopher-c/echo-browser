// Responsabilite : selecteur a segments — un seul choix, le fond actif glisse d'un segment a l'autre.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { PANEL } from './motion'

export interface Segment<T extends string> {
  id: T
  label: string
  /** Petite marque a droite du libelle (compteur, pastille). */
  badge?: ReactElement | null
}

export interface SegmentedProps<T extends string> {
  segments: Array<Segment<T>>
  value: T
  onChange: (next: T) => void
  /** Identifiant unique du groupe, pour que le fond glissant ne se confonde pas avec un autre. */
  name: string
}

export function Segmented<T extends string>({ segments, value, onChange, name }: SegmentedProps<T>): ReactElement {
  return (
    <div role="tablist" className="flex h-7 items-stretch gap-0.5 rounded-row bg-field p-0.5 shadow-field">
      {segments.map((segment) => {
        const active = segment.id === value
        return (
          <button
            key={segment.id}
            type="button"
            role="tab"
            aria-selected={active}
            onClick={() => onChange(segment.id)}
            className={`relative isolate flex flex-1 items-center justify-center gap-1.5 rounded-[6px] px-2
              text-[11.5px] transition-colors duration-100
              ${active ? 'text-ink' : 'text-ink-muted hover:text-ink'}`}
          >
            {active && (
              <motion.span
                layoutId={`segment-${name}`}
                transition={PANEL}
                className="absolute inset-0 -z-10 rounded-[6px] bg-card shadow-card"
              />
            )}
            {segment.label}
            {segment.badge}
          </button>
        )
      })}
    </div>
  )
}
