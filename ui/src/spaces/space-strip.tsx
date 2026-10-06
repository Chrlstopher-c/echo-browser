// Responsabilite : bande des espaces au pied de la barre — un point par teinte (le courant etire en trait) et
// la bascule clair/sombre. Le nom de l'espace n'apparait qu'au survol : la bande doit se faire oublier.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { IconMoon, IconSun } from '../shared/design/icons'
import { PANEL } from '../shared/design/motion'
import { buildSpace, HUES, type Scheme, type SpaceId } from './space-palette'

export interface SpaceStripProps {
  current: SpaceId
  scheme: Scheme
  onSelect: (id: SpaceId) => void
  onToggleScheme: () => void
}

function HueDot(props: {
  id: SpaceId; name: string; active: boolean; scheme: Scheme; onSelect: (id: SpaceId) => void
}): ReactElement {
  const { id, name, active, scheme, onSelect } = props
  return (
    <button
      type="button"
      role="radio"
      aria-checked={active}
      title={name}
      aria-label={`Espace ${name}`}
      onClick={() => onSelect(id)}
      className="grid h-4 place-items-center px-0.5"
    >
      <motion.span
        layout
        transition={PANEL}
        style={{ backgroundColor: active ? buildSpace(id, scheme).tokens.tint : undefined }}
        className={`block h-1.5 rounded-full ${active ? 'w-4' : 'w-1.5 bg-ink/20 hover:bg-ink/40'}`}
      />
    </button>
  )
}

export function SpaceStrip({ current, scheme, onSelect, onToggleScheme }: SpaceStripProps): ReactElement {
  return (
    <div className="flex h-7 items-center justify-between">
      <div role="radiogroup" aria-label="Espaces" className="flex h-5 items-center gap-1.5">
        {HUES.map((hue) => (
          <HueDot key={hue.id} id={hue.id} name={hue.name} active={hue.id === current} scheme={scheme}
            onSelect={onSelect} />
        ))}
      </div>
      <button
        type="button"
        title={scheme === 'dark' ? 'Thème clair' : 'Thème sombre'}
        aria-label={scheme === 'dark' ? 'Passer au thème clair' : 'Passer au thème sombre'}
        onClick={onToggleScheme}
        className="grid size-7 place-items-center rounded-full text-ink-muted transition-shadow duration-150
          hover:text-ink hover:shadow-card active:shadow-pressed"
      >
        {scheme === 'dark' ? <IconSun size={14} /> : <IconMoon size={14} />}
      </button>
    </div>
  )
}
