// Responsabilite : les onglets epingles — pastilles sans texte en tete de barre, une colonne en rail.

import { AnimatePresence, motion } from 'framer-motion'
import type { MouseEvent, ReactElement } from 'react'
import type { TabId, TabView } from '../shared/contract'
import { PANEL, QUICK } from '../shared/design/motion'
import { fallbackTitle } from '../shared/url-shape'
import { AudioBars, TabMark } from './tab-mark'

export interface PinnedGridProps {
  pinned: TabView[]
  activeId: TabId | null
  compact: boolean
  onSelect: (id: TabId) => void
  onContextMenu: (id: TabId) => (event: MouseEvent) => void
}

function PinnedTile(props: {
  tab: TabView; active: boolean; compact: boolean; onSelect: () => void; onContextMenu: (event: MouseEvent) => void
}): ReactElement {
  const { tab, active, compact, onSelect, onContextMenu } = props
  const title = tab.title.length > 0 ? tab.title : fallbackTitle(tab.url)
  return (
    <motion.button
      layout
      type="button"
      title={title}
      aria-label={title}
      aria-pressed={active}
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.9 }}
      transition={QUICK}
      onClick={onSelect}
      onContextMenu={onContextMenu}
      className={`relative isolate grid place-items-center rounded-tile transition-colors duration-100
        ${compact ? 'h-8 w-full' : 'size-9'}
        ${active ? 'text-ink' : 'bg-ink/5 text-ink-muted hover:bg-hover hover:text-ink'}`}
    >
      {active && (
        <motion.span layoutId="onglet-actif" transition={PANEL}
          className="absolute inset-0 -z-10 rounded-tile bg-card shadow-card" />
      )}
      <TabMark tab={tab} size={compact ? 16 : 18} />
      {tab.audible && !tab.asleep && (
        <span className="absolute top-1 right-1 text-guard"><AudioBars className="h-2" /></span>
      )}
    </motion.button>
  )
}

export function PinnedGrid(props: PinnedGridProps): ReactElement | null {
  const { pinned, activeId, compact, onSelect, onContextMenu } = props
  if (pinned.length === 0) return null
  return (
    <div className={`flex gap-1.5 ${compact ? 'flex-col' : 'flex-wrap'}`}>
      <AnimatePresence initial={false}>
        {pinned.map((tab) => (
          <PinnedTile
            key={tab.id}
            tab={tab}
            active={tab.id === activeId}
            compact={compact}
            onSelect={() => onSelect(tab.id)}
            onContextMenu={onContextMenu(tab.id)}
          />
        ))}
      </AnimatePresence>
    </div>
  )
}
