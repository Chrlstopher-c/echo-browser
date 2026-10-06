// Responsabilite : choix de l'espace dans les reglages — une pastille par teinte, la courante cochee.

import type { ReactElement } from 'react'
import { IconCheck } from '../shared/design/icons'
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
            style={{
              backgroundImage: `linear-gradient(135deg, ${space.tokens.glow}, ${space.tokens.shell})`,
              color: space.tokens.ink,
              boxShadow: active ? `0 0 0 1.5px ${space.tokens.tint}` : undefined,
            }}
            className={`grid aspect-square place-items-center rounded-full ring-1 transition-[transform,box-shadow]
              duration-150 ${active ? 'ring-transparent' : 'ring-hairline hover:scale-105'}`}
          >
            {active && <IconCheck size={12} />}
          </button>
        )
      })}
    </div>
  )
}
