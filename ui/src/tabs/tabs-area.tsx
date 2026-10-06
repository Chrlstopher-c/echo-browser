// Responsabilite : la zone des onglets — epingles au-dessus, liste en dessous, menu contextuel commun.

import { useMemo, type ReactElement } from 'react'
import type { TabId, TabView } from '../shared/contract'
import { PinnedGrid } from './pinned-grid'
import { TabList } from './tab-list'
import { TabMenu } from './tab-menu'
import type { TabActions } from './use-tab-actions'
import type { TabMenuController } from './use-tab-menu'

export interface TabsAreaProps {
  tabs: TabView[]
  activeId: TabId | null
  actions: TabActions
  menu: TabMenuController
  compact: boolean
}

export function TabsArea({ tabs, activeId, actions, menu, compact }: TabsAreaProps): ReactElement {
  const pinned = useMemo(() => tabs.filter((tab) => tab.pinned), [tabs])
  const loose = useMemo(() => tabs.filter((tab) => !tab.pinned), [tabs])
  return (
    <div className="flex flex-col gap-3">
      <PinnedGrid
        pinned={pinned}
        activeId={activeId}
        compact={compact}
        onSelect={actions.select}
        onContextMenu={menu.openFor}
      />
      {pinned.length > 0 && loose.length > 0 && <div className="mx-1 border-t border-hairline" />}
      <TabList all={tabs} loose={loose} activeId={activeId} compact={compact} actions={actions} menu={menu} />
      <TabMenu tabs={tabs} controller={menu} actions={actions} activeId={activeId} />
    </div>
  )
}
