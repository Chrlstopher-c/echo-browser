// Responsabilite : un onglet de la liste — marque, titre tronque, son, fermeture au survol.
// L'onglet actif est une carte posee : le fond glisse d'une ligne a l'autre. Saisi, il se souleve.

import { motion, Reorder, useDragControls } from 'framer-motion'
import type { MouseEvent, PointerEvent, ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import { IconClose } from '../shared/design/icons'
import { PANEL, QUICK } from '../shared/design/motion'
import { fallbackTitle } from '../shared/url-shape'
import { AudioBars, TabMark } from './tab-mark'

export interface TabRowProps {
  tab: TabView
  active: boolean
  compact: boolean
  onSelect: () => void
  onClose: () => void
  onContextMenu: (event: MouseEvent) => void
  onDragEnd: () => void
}

const ROW_MOTION = {
  initial: { opacity: 0, height: 0 },
  animate: { opacity: 1, height: 32 },
  exit: { opacity: 0, height: 0 },
  whileDrag: { scale: 1.02, boxShadow: 'var(--shadow-lift)', zIndex: 5 },
}

function CloseAction({ onClose }: { onClose: () => void }): ReactElement {
  return (
    <button
      type="button"
      aria-label="Fermer l'onglet"
      title="Fermer l'onglet"
      onPointerDown={(event) => event.stopPropagation()}
      onClick={(event) => {
        event.stopPropagation()
        onClose()
      }}
      className="grid size-5 shrink-0 place-items-center rounded-md text-ink-faint opacity-0
        transition-opacity duration-100 group-hover:opacity-100 hover:bg-ink/10 hover:text-ink
        focus-visible:opacity-100"
    >
      <IconClose size={12} />
    </button>
  )
}

function RowBody({ tab, title, onClose }: { tab: TabView; title: string; onClose: () => void }): ReactElement {
  return (
    <>
      <span className={`min-w-0 flex-1 truncate text-[12.5px] leading-none ${tab.asleep ? 'text-ink-faint' : ''}`}>
        {title}
      </span>
      {tab.audible && !tab.asleep && (
        <AudioBars className="shrink-0 text-guard transition-opacity duration-100 group-hover:opacity-0" />
      )}
      <span className={`shrink-0 ${tab.audible ? '-ml-5' : ''}`}>
        <CloseAction onClose={onClose} />
      </span>
    </>
  )
}

function ActiveBackdrop(): ReactElement {
  return (
    <motion.span layoutId="onglet-actif" transition={PANEL} className="absolute inset-0 -z-10 rounded-row shadow-card">
      <span className="absolute top-1/2 left-1.5 h-3.5 w-[3px] -translate-y-1/2 rounded-full bg-tint" />
    </motion.span>
  )
}

export function TabRow(props: TabRowProps): ReactElement {
  const { tab, active, compact, onSelect, onClose, onContextMenu, onDragEnd } = props
  const title = tab.title.length > 0 ? tab.title : fallbackTitle(tab.url)
  const controls = useDragControls()
  const onPointerDown = (event: PointerEvent): void => {
    if (event.button !== 0) return
    onSelect()
    if (!compact) controls.start(event)
  }
  return (
    <Reorder.Item
      value={tab}
      dragListener={false}
      dragControls={controls}
      onDragEnd={onDragEnd}
      {...ROW_MOTION}
      transition={QUICK}
      onPointerDown={onPointerDown}
      onContextMenu={onContextMenu}
      title={title}
      className={`group relative isolate flex items-center gap-2.5 rounded-row
        ${compact ? 'justify-center px-0' : 'pr-1.5 pl-4'}
        ${active ? 'text-ink' : 'text-ink-muted hover:bg-hover hover:text-ink'}`}
    >
      {active && <ActiveBackdrop />}
      <TabMark tab={tab} size={16} />
      {!compact && <RowBody tab={tab} title={title} onClose={onClose} />}
    </Reorder.Item>
  )
}
