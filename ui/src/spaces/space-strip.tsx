// Responsabilite : bande des espaces au pied de la barre — six points, le courant etire en trait.
// Le nom de l'espace n'apparait qu'au survol : la bande doit se faire oublier.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { PANEL } from '../shared/design/motion'
import { SPACES, type SpaceId } from './space-palette'

export interface SpaceStripProps {
  current: SpaceId
  onSelect: (id: SpaceId) => void
}

export function SpaceStrip({ current, onSelect }: SpaceStripProps): ReactElement {
  return (
    <div role="radiogroup" aria-label="Espaces" className="flex h-5 items-center justify-center gap-1.5">
      {SPACES.map((space) => {
        const active = space.id === current
        return (
          <button
            key={space.id}
            type="button"
            role="radio"
            aria-checked={active}
            title={space.name}
            aria-label={`Espace ${space.name}`}
            onClick={() => onSelect(space.id)}
            className="grid h-4 place-items-center px-0.5"
          >
            <motion.span
              layout
              transition={PANEL}
              style={{ backgroundColor: active ? space.tokens.tint : undefined }}
              className={`block h-1.5 rounded-full ${active ? 'w-4' : 'w-1.5 bg-ink/20 hover:bg-ink/40'}`}
            />
          </button>
        )
      })}
    </div>
  )
}
