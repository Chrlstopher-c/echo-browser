// Responsabilite : liste verticale des onglets libres, reordonnable, et bouton de nouvel onglet.
// L'ordre local suit le coeur ; pendant un glisser, il vit ici, puis `moveTab` part au relachement.

import { AnimatePresence, Reorder } from 'framer-motion'
import { useEffect, useState, type ReactElement } from 'react'
import type { TabId, TabView } from '../shared/contract'
import { IconPlus } from '../shared/design/icons'
import { folderAt } from './drop-target'
import { TabRow, type DropPoint } from './tab-row'
import { targetIndex, type TabActions } from './use-tab-actions'
import type { TabMenuController } from './use-tab-menu'

export interface TabListProps {
  all: TabView[]
  loose: TabView[]
  activeId: TabId | null
  compact: boolean
  actions: TabActions
  menu: TabMenuController
  /** Onglet lache sur un dossier. */
  onFolder: (id: TabId, folder: string) => void
}

function NewTabRow({ compact, onNew }: { compact: boolean; onNew: () => void }): ReactElement {
  return (
    <button
      type="button"
      onClick={onNew}
      aria-label="Nouvel onglet"
      title="Nouvel onglet (Ctrl+T)"
      className={`flex h-8 items-center gap-2.5 rounded-row text-ink-faint transition-colors duration-100
        hover:bg-hover hover:text-ink-muted ${compact ? 'justify-center' : 'pl-4 pr-2.5'}`}
    >
      <IconPlus size={14} />
      {!compact && <span className="text-[12.5px] leading-none">Nouvel onglet</span>}
    </button>
  )
}

/** Ordre local pendant le glisser, realigne sur le coeur des qu'il parle. */
function useLocalOrder(loose: TabView[]): [TabView[], (next: TabView[]) => void] {
  const [order, setOrder] = useState(loose)
  useEffect(() => setOrder(loose), [loose])
  return [order, setOrder]
}

/** Lacher d'un onglet libre : sur un dossier il y entre, sinon il prend sa nouvelle place dans la liste. */
function dropTab(drop: {
  id: TabId; point: DropPoint; all: TabView[]; order: TabView[]; actions: TabActions
  onFolder: (id: TabId, folder: string) => void
}): void {
  const { id, point, all, order, actions, onFolder } = drop
  const folder = folderAt(point)
  if (typeof folder === 'string') return onFolder(id, folder)
  const to = targetIndex(all, order, id)
  const from = all.findIndex((tab) => tab.id === id)
  if (to !== -1 && to !== from) actions.move(id, to)
}

export function TabList(props: TabListProps): ReactElement {
  const { all, loose, activeId, compact, actions, menu, onFolder } = props
  const [order, setOrder] = useLocalOrder(loose)
  const onDragEnd = (id: TabId) => (point: DropPoint): void => dropTab({ id, point, all, order, actions, onFolder })

  return (
    <div data-loose-tabs className="flex flex-col gap-0.5">
      <Reorder.Group axis="y" values={order} onReorder={setOrder} className="flex flex-col gap-0.5"
        role="tablist" aria-orientation="vertical" aria-label="Onglets">
        <AnimatePresence initial={false}>
          {order.map((tab) => (
            <TabRow
              key={tab.id}
              tab={tab}
              active={tab.id === activeId}
              compact={compact}
              onSelect={() => actions.select(tab.id)}
              onClose={() => actions.close(tab.id)}
              onWarm={() => actions.warm(tab.id)}
              onContextMenu={menu.openFor(tab.id)}
              onDragEnd={onDragEnd(tab.id)}
            />
          ))}
        </AnimatePresence>
      </Reorder.Group>
      <NewTabRow compact={compact} onNew={() => actions.newTab()} />
    </div>
  )
}
