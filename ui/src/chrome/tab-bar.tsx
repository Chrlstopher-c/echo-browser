// Responsabilite : bande d'onglets — liste, selection, fermeture, nouvel onglet.

import { AnimatePresence } from 'framer-motion'
import type { ReactElement } from 'react'
import type { TabId, TabView } from '../shared/contract'
import { ChromeButton } from '../shared/design/chrome-button'
import { TAB_STRIP_HEIGHT } from '../shared/design/geometry'
import { IconPlus } from '../shared/design/icons'
import { TabItem } from './tab-item'

export interface TabBarProps {
  tabs: TabView[]
  activeId: TabId | null
  onSelect: (id: TabId) => void
  onClose: (id: TabId) => void
  onNew: () => void
}

export function TabBar({ tabs, activeId, onSelect, onClose, onNew }: TabBarProps): ReactElement {
  return (
    <div
      style={{ height: TAB_STRIP_HEIGHT }}
      className="flex shrink-0 items-end gap-px bg-shell px-1.5 pb-1"
    >
      <div className="flex min-w-0 flex-1 items-end gap-px">
        <AnimatePresence initial={false}>
          {tabs.map((tab) => (
            <TabItem
              key={tab.id}
              tab={tab}
              active={tab.id === activeId}
              onSelect={() => onSelect(tab.id)}
              onClose={() => onClose(tab.id)}
            />
          ))}
        </AnimatePresence>
      </div>
      <div className="pb-[1px] pl-1">
        <ChromeButton label="Nouvel onglet" onClick={onNew} compact>
          <IconPlus size={14} />
        </ChromeButton>
      </div>
    </div>
  )
}
