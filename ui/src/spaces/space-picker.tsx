// Responsabilite : choix de l'espace dans les reglages — une pastille par teinte, la courante cochee.

import type { ReactElement } from 'react'
import { buildSpace, HUES, type Scheme, type SpaceId } from './space-palette'

export interface SpacePickerProps {
  current: SpaceId
  scheme: Scheme
  onSelect: (id: SpaceId) => void
}

export function SpacePicker({ current, scheme, onSelect }: SpacePickerProps): ReactElement {
  return (
    <div className="grid grid-cols-6 gap-2 px-2 py-1">
      {HUES.map((hue) => {
        const space = buildSpace(hue.id, scheme)
        const active = space.id === current
        return (
          <button
            key={space.id}
            type="button"
            title={space.name}
            aria-label={`Espace ${space.name}`}
            aria-pressed={active}
            onClick={() => onSelect(space.id)}
            style={{ color: space.tokens.tint }}
            className={`grid aspect-square place-items-center rounded-full bg-card transition-shadow duration-150
              ${active ? 'shadow-pressed' : 'shadow-card'}`}
          >
            <span className="size-2.5 rounded-full" style={{ backgroundColor: space.tokens.tint, opacity: active ? 1 : 0.55 }} />
          </button>
        )
      })}
    </div>
  )
}
