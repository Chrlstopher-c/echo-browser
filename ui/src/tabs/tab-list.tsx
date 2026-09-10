// Responsabilite : liste verticale des onglets libres, reordonnable, et bouton de nouvel onglet.
// L'ordre local suit le coeur ; pendant un glisser, il vit ici, puis `moveTab` part au relachement.

import { AnimatePresence, Reorder } from 'framer-motion'
import { useEffect, useRef, useState, type ReactElement } from 'react'
import type { TabId, TabView } from '../shared/contract'
import { IconPlus } from '../shared/design/icons'
import { TabRow } from './tab-row'
import { targetIndex, type TabActions } from './use-tab-actions'
import type { TabMenuController } from './use-tab-menu'

export interface TabListProps {
  all: TabView[]
  loose: TabView[]
  activeId: TabId | null
  compact: boolean
  actions: TabActions
  menu: TabMenuController
}

function NewTabRow({ compact, onNew }: { compact: boolean; onNew: () => void }): ReactElement {
  return (
    <button
      type="button"
      onClick={onNew}
      aria-label="Nouvel onglet"
      title="Nouvel onglet (Ctrl+T)"
      className={`flex h-8 items-center gap-2.5 rounded-row text-ink-faint transition-colors duration-100
        hover:bg-hover hover:text-ink-muted ${compact ? 'justify-center' : 'px-2.5'}`}
    >
      <IconPlus size={14} />
      {!compact && <span className="text-[12.5px]">Nouvel onglet</span>}
    </button>
  )
}

/** Ordre local pendant le glisser, realigne sur le coeur des qu'il parle. */
function useLocalOrder(loose: TabView[]): [TabView[], (next: TabView[]) => void] {
  const [order, setOrder] = useState(loose)
  useEffect(() => setOrder(loose), [loose])
  return [order, setOrder]
}

export function TabList(props: TabListProps): ReactElement {
  const { all, loose, activeId, compact, actions, menu } = props
  const [order, setOrder] = useLocalOrder(loose)
  const dragged = useRef<TabId | null>(null)

  const onDragEnd = (id: TabId) => (): void => {
    dragged.current = null
    const to = targetIndex(all, order, id)
    const from = all.findIndex((tab) => tab.id === id)
    if (to !== -1 && to !== from) actions.move(id, to)
  }

  return (
    <div className="flex flex-col gap-0.5">
      <Reorder.Group axis="y" values={order} onReorder={setOrder} className="flex flex-col gap-0.5">
        <AnimatePresence initial={false}>
          {order.map((tab) => (
            <TabRow
              key={tab.id}
              tab={tab}
              active={tab.id === activeId}
              compact={compact}
              onSelect={() => actions.select(tab.id)}
              onClose={() => actions.close(tab.id)}
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
