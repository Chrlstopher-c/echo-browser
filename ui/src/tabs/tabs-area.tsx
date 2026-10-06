// Responsabilite : la zone des onglets — epingles au-dessus, liste en dessous, menu contextuel commun.

import { useMemo, type ReactElement } from 'react'
import type { TabId, TabView } from '../shared/contract'
import { FolderSection } from './folder-section'
import { PinnedGrid } from './pinned-grid'
import { TabList } from './tab-list'
import { TabMenu } from './tab-menu'
import type { TabActions } from './use-tab-actions'
import type { FolderActions } from './use-folders'
import type { TabMenuController } from './use-tab-menu'

export interface TabsAreaProps {
  tabs: TabView[]
  activeId: TabId | null
  actions: TabActions
  menu: TabMenuController
  folders: FolderActions
  compact: boolean
}

export function TabsArea({ tabs, activeId, actions, menu, folders, compact }: TabsAreaProps): ReactElement {
  const pinned = useMemo(() => tabs.filter((tab) => tab.pinned), [tabs])
  const known = useMemo(() => new Set(folders.folders.map((folder) => folder.id)), [folders.folders])
  const loose = useMemo(
    () => tabs.filter((tab) => !tab.pinned && (tab.folder === null || !known.has(tab.folder))),
    [tabs, known],
  )
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
      {!compact && <p className="intitule px-4">Onglets</p>}
      {!compact && folders.folders.map((folder) => (
        <FolderSection
          key={folder.id}
          folder={folder}
          members={tabs.filter((tab) => !tab.pinned && tab.folder === folder.id)}
          activeId={activeId}
          actions={actions}
          folders={folders}
          menu={menu}
        />
      ))}
      <TabList all={tabs} loose={loose} activeId={activeId} compact={compact} actions={actions} menu={menu} />
      <TabMenu tabs={tabs} controller={menu} actions={actions} folders={folders} activeId={activeId} />
    </div>
  )
}
