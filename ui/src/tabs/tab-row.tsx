// Responsabilite : un onglet de la liste — favicon, titre tronque, epingler et fermer au survol.
// L'onglet actif est une carte posee : le fond glisse d'une ligne a l'autre.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import { IconClose, IconPin } from '../shared/design/icons'
import { SiteMark } from '../shared/design/site-mark'
import { Spinner } from '../shared/design/spinner'
import { fallbackTitle } from '../shared/url-shape'
import { CHROME_EASE } from '../sidebar/sidebar-geometry'

export interface TabRowProps {
  tab: TabView
  active: boolean
  compact: boolean
  onSelect: () => void
  onClose: () => void
  onPin: () => void
}

interface HoverActionProps {
  label: string
  onClick: () => void
  children: ReactElement
}

function HoverAction({ label, onClick, children }: HoverActionProps): ReactElement {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onPointerDown={(event) => event.stopPropagation()}
      onClick={(event) => {
        event.stopPropagation()
        onClick()
      }}
      className="grid size-5 shrink-0 place-items-center rounded-md text-ink-faint opacity-0
        transition-opacity duration-100 group-hover:opacity-100 hover:bg-ink/10 hover:text-ink
        focus-visible:opacity-100"
    >
      {children}
    </button>
  )
}

function RowBody({ title, onPin, onClose }: { title: string; onPin: () => void; onClose: () => void }): ReactElement {
  return (
    <>
      <span className="min-w-0 flex-1 truncate text-[12.5px]">{title}</span>
      <HoverAction label="Ajouter aux essentiels" onClick={onPin}>
        <IconPin size={12} />
      </HoverAction>
      <HoverAction label="Fermer l'onglet" onClick={onClose}>
        <IconClose size={12} />
      </HoverAction>
    </>
  )
}

export function TabRow({ tab, active, compact, onSelect, onClose, onPin }: TabRowProps): ReactElement {
  const title = tab.title.length > 0 ? tab.title : fallbackTitle(tab.url)
  return (
    <motion.div
      layout="position"
      initial={{ opacity: 0, height: 0 }}
      animate={{ opacity: 1, height: 32 }}
      exit={{ opacity: 0, height: 0 }}
      transition={{ duration: 0.18, ease: CHROME_EASE }}
      onPointerDown={onSelect}
      title={title}
      className={`group relative isolate flex items-center gap-2.5 overflow-hidden rounded-row
        ${compact ? 'justify-center px-0' : 'px-2.5'}
        ${active ? 'text-ink' : 'text-ink-muted hover:bg-hover hover:text-ink'}`}
    >
      {active && (
        <motion.span
          layoutId="onglet-actif"
          transition={{ duration: 0.2, ease: CHROME_EASE }}
          className="absolute inset-0 -z-10 rounded-row bg-card shadow-card"
        />
      )}
      {tab.loading ? <Spinner size={16} /> : <SiteMark url={tab.url} favicon={tab.favicon} size={16} />}
      {!compact && <RowBody title={title} onPin={onPin} onClose={onClose} />}
    </motion.div>
  )
}
