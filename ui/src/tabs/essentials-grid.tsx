// Responsabilite : grille des essentiels — deux pastilles par ligne, une colonne en rail.

import { AnimatePresence } from 'framer-motion'
import type { ReactElement } from 'react'
import type { TabId, TabView } from '../shared/contract'
import { EssentialTile } from './essential-tile'
import { ESSENTIALS_COLUMNS, type Essential } from './essentials-model'

export interface EssentialsGridProps {
  essentials: Essential[]
  represented: Map<string, TabView>
  activeId: TabId | null
  compact: boolean
  onSelect: (id: TabId) => void
  onOpen: (url: string) => void
  onUnpin: (host: string) => void
}

export function EssentialsGrid(props: EssentialsGridProps): ReactElement | null {
  const { essentials, represented, activeId, compact } = props
  if (essentials.length === 0) return null
  const columns = compact ? 1 : ESSENTIALS_COLUMNS
  return (
    <div style={{ gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` }} className="grid gap-2">
      <AnimatePresence initial={false}>
        {essentials.map((item) => {
          const tab = represented.get(item.host)
          return (
            <EssentialTile
              key={item.host}
              item={item}
              active={tab !== undefined && tab.id === activeId}
              loading={tab?.loading ?? false}
              compact={compact}
              onSelect={() => (tab === undefined ? props.onOpen(item.url) : props.onSelect(tab.id))}
              onUnpin={() => props.onUnpin(item.host)}
            />
          )
        })}
      </AnimatePresence>
    </div>
  )
}
