// Responsabilite : bande des profils au pied de la barre — un point par profil, a sa teinte (le courant etire en
// trait), un « + » pour en creer un, et la bascule clair/sombre. Les noms n'apparaissent qu'au survol.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { IconMoon, IconPlus, IconSun } from '../shared/design/icons'
import { PANEL } from '../shared/design/motion'
import { buildSpace, type Scheme, type SpaceId } from './space-palette'
import type { ProfileEntry } from './use-profile-names'

export interface SpaceStripProps {
  current: string
  scheme: Scheme
  profiles: ProfileEntry[]
  onSelect: (id: string) => void
  /** Cree un profil (et y passe). */
  onCreate: () => void
  onToggleScheme: () => void
}

function HueDot(props: {
  id: string; hue: SpaceId; name: string; active: boolean; scheme: Scheme; onSelect: (id: string) => void
}): ReactElement {
  const { id, hue, name, active, scheme, onSelect } = props
  return (
    <button
      type="button"
      role="radio"
      aria-checked={active}
      title={name}
      aria-label={`Profil ${name}`}
      onClick={() => onSelect(id)}
      className="grid h-4 place-items-center px-0.5"
    >
      <motion.span
        layout
        transition={PANEL}
        style={{ backgroundColor: active ? buildSpace(id, scheme, hue).tokens.tint : undefined }}
        className={`block h-1.5 rounded-full ${active ? 'w-4' : 'w-1.5 bg-ink/20 hover:bg-ink/40'}`}
      />
    </button>
  )
}

export function SpaceStrip(props: SpaceStripProps): ReactElement {
  const { current, scheme, profiles, onSelect, onCreate, onToggleScheme } = props
  return (
    <div className="flex h-7 items-center justify-between">
      <div role="radiogroup" aria-label="Profils" className="flex h-5 min-w-0 items-center gap-1.5 overflow-hidden">
        {profiles.map((p) => (
          <HueDot key={p.id} id={p.id} hue={p.hue} name={p.name} active={p.id === current} scheme={scheme}
            onSelect={onSelect} />
        ))}
        <button type="button" title="Nouveau profil" aria-label="Nouveau profil" onClick={onCreate}
          className="grid size-4 place-items-center rounded-full text-ink-faint hover:text-ink">
          <IconPlus size={10} />
        </button>
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
