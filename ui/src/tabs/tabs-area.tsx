// Responsabilite : la zone des onglets — essentiels au-dessus, liste en dessous, en colonne ou en rail.

import type { ReactElement } from 'react'
import type { TabId } from '../shared/contract'
import type { SidebarActions } from '../sidebar/use-sidebar-actions'
import { EssentialsGrid } from './essentials-grid'
import { TabList } from './tab-list'
import type { EssentialsController } from './use-essentials'

export interface TabsAreaProps {
  essentials: EssentialsController
  activeId: TabId | null
  actions: SidebarActions
  compact: boolean
}

export function TabsArea({ essentials, activeId, actions, compact }: TabsAreaProps): ReactElement {
  return (
    <div className="flex flex-col gap-3">
      <EssentialsGrid
        essentials={essentials.essentials}
        represented={essentials.represented}
        activeId={activeId}
        compact={compact}
        onSelect={actions.selectTab}
        onOpen={(url) => actions.newTab(url)}
        onUnpin={essentials.unpin}
      />
      <TabList
        tabs={essentials.loose}
        activeId={activeId}
        compact={compact}
        onSelect={actions.selectTab}
        onClose={actions.closeTab}
        onPin={essentials.pin}
        onNew={() => actions.newTab()}
      />
    </div>
  )
}
