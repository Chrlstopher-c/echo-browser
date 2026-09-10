// Responsabilite : pastille d'un essentiel — favicon centre, sans texte, retrait au survol.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { IconUnpin } from '../shared/design/icons'
import { SiteMark } from '../shared/design/site-mark'
import { Spinner } from '../shared/design/spinner'
import { CHROME_EASE } from '../sidebar/sidebar-geometry'
import type { Essential } from './essentials-model'

export interface EssentialTileProps {
  item: Essential
  active: boolean
  loading: boolean
  compact: boolean
  onSelect: () => void
  onUnpin: () => void
}

function UnpinButton({ onClick }: { onClick: () => void }): ReactElement {
  return (
    <button
      type="button"
      aria-label="Retirer des essentiels"
      title="Retirer des essentiels"
      onClick={onClick}
      className="absolute top-1 right-1 grid size-5 place-items-center rounded-md bg-shell/80 text-ink-faint
        opacity-0 transition-opacity duration-100 group-hover:opacity-100 hover:text-ink focus-visible:opacity-100"
    >
      <IconUnpin size={11} />
    </button>
  )
}

export function EssentialTile(props: EssentialTileProps): ReactElement {
  const { item, active, loading, compact, onSelect, onUnpin } = props
  const size = compact ? 16 : 24
  const surface = active ? 'bg-card shadow-card' : 'bg-ink/5 hover:bg-hover'
  return (
    <motion.div
      layout="position"
      initial={{ opacity: 0, scale: 0.94 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.94 }}
      transition={{ duration: 0.16, ease: CHROME_EASE }}
      className="group relative"
    >
      <button
        type="button"
        title={item.title.length > 0 ? item.title : item.host}
        aria-label={item.title.length > 0 ? item.title : item.host}
        aria-pressed={active}
        onClick={onSelect}
        className={`grid w-full place-items-center transition-colors duration-100
          ${compact ? 'h-8 rounded-row' : 'aspect-square rounded-tile'} ${surface}`}
      >
        {loading ? <Spinner size={size} /> : <SiteMark url={item.url} favicon={item.favicon} size={size} />}
      </button>
      {!compact && <UnpinButton onClick={onUnpin} />}
    </motion.div>
  )
}
