// Responsabilite : liste verticale des onglets et bouton de nouvel onglet.

import { AnimatePresence } from 'framer-motion'
import type { ReactElement } from 'react'
import type { TabId, TabView } from '../shared/contract'
import { IconPlus } from '../shared/design/icons'
import { TabRow } from './tab-row'

export interface TabListProps {
  tabs: TabView[]
  activeId: TabId | null
  compact: boolean
  onSelect: (id: TabId) => void
  onClose: (id: TabId) => void
  onPin: (tab: TabView) => void
  onNew: () => void
}

function NewTabRow({ compact, onNew }: { compact: boolean; onNew: () => void }): ReactElement {
  return (
    <button
      type="button"
      onClick={onNew}
      aria-label="Nouvel onglet"
      title="Nouvel onglet"
      className={`flex h-8 items-center gap-2.5 rounded-row text-ink-faint transition-colors duration-100
        hover:bg-hover hover:text-ink-muted ${compact ? 'justify-center' : 'px-2.5'}`}
    >
      <IconPlus size={14} />
      {!compact && <span className="text-[12.5px]">Nouvel onglet</span>}
    </button>
  )
}

export function TabList(props: TabListProps): ReactElement {
  const { tabs, activeId, compact, onSelect, onClose, onPin, onNew } = props
  return (
    <div className="flex flex-col gap-0.5">
      <AnimatePresence initial={false}>
        {tabs.map((tab) => (
          <TabRow
            key={tab.id}
            tab={tab}
            active={tab.id === activeId}
            compact={compact}
            onSelect={() => onSelect(tab.id)}
            onClose={() => onClose(tab.id)}
            onPin={() => onPin(tab)}
          />
        ))}
      </AnimatePresence>
      <NewTabRow compact={compact} onNew={onNew} />
    </div>
  )
}
