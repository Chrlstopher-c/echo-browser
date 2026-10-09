// Responsabilite : un dossier d'onglets dans la barre — en-tete (plier, renommer) et onglets rangés dedans.

import { Reorder } from 'framer-motion'
import { useState, type KeyboardEvent, type ReactElement } from 'react'
import type { TabId, TabView } from '../shared/contract'
import { IconChevron, IconFolder } from '../shared/design/icons'
import { folderAt } from './drop-target'
import { TabRow } from './tab-row'
import type { Folder, FolderActions } from './use-folders'
import type { TabActions } from './use-tab-actions'
import type { TabMenuController } from './use-tab-menu'

export interface FolderSectionProps {
  folder: Folder
  members: TabView[]
  activeId: TabId | null
  actions: TabActions
  folders: FolderActions
  menu: TabMenuController
}

function NameField({ folder, folders }: { folder: Folder; folders: FolderActions }): ReactElement {
  const [value, setValue] = useState(folder.name)
  const commit = (): void => {
    folders.rename(folder.id, value)
    folders.edit(null)
  }
  const onKey = (event: KeyboardEvent): void => {
    if (event.key === 'Enter') commit()
    if (event.key === 'Escape') folders.edit(null)
    event.stopPropagation()
  }
  return (
    <input
      autoFocus
      value={value}
      aria-label="Nom du dossier"
      onChange={(event) => setValue(event.target.value)}
      onFocus={(event) => event.target.select()}
      onBlur={commit}
      onKeyDown={onKey}
      className="h-6 min-w-0 flex-1 rounded-md bg-field px-1.5 text-[12.5px] text-ink shadow-field outline-none"
    />
  )
}

function Header(props: FolderSectionProps): ReactElement {
  const { folder, members, folders, menu } = props
  const editing = folders.editing === folder.id
  return (
    <div
      role="button"
      data-folder-id={folder.id}
      tabIndex={0}
      aria-expanded={!folder.collapsed}
      onClick={() => !editing && folders.toggle(folder.id)}
      onDoubleClick={() => folders.edit(folder.id)}
      onKeyDown={(event) => event.key === 'Enter' && folders.toggle(folder.id)}
      onContextMenu={menu.openFolder(folder.id)}
      className="flex h-8 cursor-default items-center gap-2.5 rounded-row pr-2.5 pl-4 text-ink-muted
        transition-colors duration-100 hover:bg-hover hover:text-ink"
    >
      <IconFolder size={16} />
      {editing ? (
        <NameField folder={folder} folders={folders} />
      ) : (
        <span className="min-w-0 flex-1 truncate text-[12.5px] leading-none">{folder.name}</span>
      )}
      <span className="numerique text-[10.5px] text-ink-faint">{members.length}</span>
      <span className={`text-ink-faint transition-transform duration-150 ${folder.collapsed ? '' : 'rotate-90'}`}>
        <IconChevron size={12} />
      </span>
    </div>
  )
}

export function FolderSection(props: FolderSectionProps): ReactElement {
  const { folder, members, activeId, actions, folders, menu } = props
  return (
    <div data-folder-id={folder.id} className="flex flex-col gap-0.5">
      <Header {...props} />
      {!folder.collapsed && (
        <Reorder.Group axis="y" values={members} onReorder={() => undefined} className="flex flex-col gap-0.5 pl-3"
          role="tablist" aria-orientation="vertical" aria-label={`Dossier ${folder.name}`}>
          {members.map((tab) => (
            <TabRow
              key={tab.id}
              tab={tab}
              active={tab.id === activeId}
              compact={false}
              onSelect={() => actions.select(tab.id)}
              onClose={() => actions.close(tab.id)}
              onWarm={() => actions.warm(tab.id)}
              onContextMenu={menu.openFor(tab.id)}
              onDragEnd={(point) => {
                const target = folderAt(point)
                if (target !== undefined && target !== folder.id) folders.assign(tab.id, target)
              }}
            />
          ))}
        </Reorder.Group>
      )}
    </div>
  )
}
