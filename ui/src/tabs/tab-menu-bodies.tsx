// Responsabilite : le contenu du menu contextuel selon la cible — un onglet, un dossier, le fond de la liste.

import type { ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import { formatZoom } from '../shared/format'
import {
  IconClose, IconFolder, IconMoon, IconPin, IconPlus, IconReload, IconStar, IconTrash, IconUnpin,
} from '../shared/design/icons'
import { Stepper } from '../shared/design/stepper'
import type { FolderActions } from './use-folders'
import type { TabActions } from './use-tab-actions'
import { MenuItem, MenuSeparator } from './menu-item'

interface Common {
  actions: TabActions
  folders: FolderActions
  close: () => void
}

function ZoomRow({ tab, actions }: { tab: TabView; actions: TabActions }): ReactElement {
  return (
    <div className="flex h-9 items-center justify-between gap-2 px-2">
      <span className="text-[12px] text-ink">Zoom</span>
      <Stepper
        value={Math.round(tab.zoom * 100)}
        min={50}
        max={300}
        step={10}
        unit="%"
        label="Zoom"
        render={(value) => formatZoom(value / 100)}
        onChange={(value) => actions.setZoom(tab.id, value / 100)}
      />
    </div>
  )
}

interface FolderItemsProps {
  tab: TabView
  folders: FolderActions
  close: () => void
}

function FolderItems({ tab, folders, close }: FolderItemsProps): ReactElement {
  const run = (action: () => void) => (): void => {
    action()
    close()
  }
  return (
    <>
      {folders.folders
        .filter((folder) => folder.id !== tab.folder)
        .map((folder) => (
          <MenuItem key={folder.id} icon={<IconFolder size={13} />} label={`Ranger dans « ${folder.name} »`}
            onClick={run(() => folders.assign(tab.id, folder.id))} />
        ))}
      {tab.folder !== null && (
        <MenuItem icon={<IconFolder size={13} />} label="Sortir du dossier"
          onClick={run(() => folders.assign(tab.id, null))} />
      )}
      <MenuItem icon={<IconPlus size={13} />} label="Nouveau dossier avec cet onglet"
        onClick={run(() => folders.create(undefined, tab.id))} />
    </>
  )
}

export function TabBody(props: Common & { tab: TabView; others: TabView[]; activeId: number | null }): ReactElement {
  const { tab, others, activeId, actions, folders, close } = props
  const run = (action: () => void) => (): void => {
    action()
    close()
  }
  return (
    <>
      <MenuItem
        icon={tab.pinned ? <IconUnpin size={13} /> : <IconPin size={13} />}
        label={tab.pinned ? 'Détacher' : 'Épingler'}
        onClick={run(() => actions.pin(tab.id, !tab.pinned))}
      />
      <MenuItem icon={<IconStar size={13} />} label="Ajouter aux favoris"
        onClick={run(() => actions.addBookmark(tab.id))} />
      <MenuItem icon={<IconReload size={13} />} label="Recharger" onClick={run(() => actions.reload(tab.id))} />
      {!tab.asleep && tab.id !== activeId && (
        <MenuItem icon={<IconMoon size={13} />} label="Endormir" onClick={run(() => actions.sleep(tab.id))} />
      )}
      <MenuSeparator />
      <FolderItems tab={tab} folders={folders} close={close} />
      <MenuSeparator />
      <ZoomRow tab={tab} actions={actions} />
      <MenuSeparator />
      <MenuItem icon={<IconClose size={13} />} label="Fermer l'onglet" danger
        onClick={run(() => actions.close(tab.id))} />
      {others.length > 0 && (
        <MenuItem icon={<IconClose size={13} />} label="Fermer les autres onglets" danger
          onClick={run(() => others.forEach((other) => actions.close(other.id)))} />
      )}
    </>
  )
}

export function FolderBody(props: Common & { id: string; members: TabView[] }): ReactElement {
  const { id, members, folders, close } = props
  const run = (action: () => void) => (): void => {
    action()
    close()
  }
  return (
    <>
      <MenuItem icon={<IconFolder size={13} />} label="Renommer" onClick={run(() => folders.edit(id))} />
      <MenuItem icon={<IconTrash size={13} />} label="Supprimer le dossier"
        onClick={run(() => folders.remove(id, members))} />
      <MenuSeparator />
      <MenuItem icon={<IconClose size={13} />} label="Fermer le dossier et ses onglets" danger
        onClick={run(() => folders.closeAll(id, members))} />
    </>
  )
}

export function AreaBody({ actions, folders, close }: Common): ReactElement {
  const run = (action: () => void) => (): void => {
    action()
    close()
  }
  return (
    <>
      <MenuItem icon={<IconPlus size={13} />} label="Nouvel onglet" onClick={run(() => actions.newTab())} />
      <MenuItem icon={<IconFolder size={13} />} label="Nouveau dossier" onClick={run(() => folders.create())} />
    </>
  )
}
