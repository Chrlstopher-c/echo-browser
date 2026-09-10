// Responsabilite : feuille des extensions — liste, interrupteur par extension, avertissement de relance.

import type { ReactElement } from 'react'
import { EmptyState } from '../shared/design/empty-state'
import { IconPuzzle } from '../shared/design/icons'
import { Toggle } from '../shared/design/toggle'
import { type Extension } from './extension-model'

export interface ExtensionsSheetProps {
  extensions: Extension[]
  onToggle: (id: string, enabled: boolean) => void
}

interface ExtensionRowProps {
  item: Extension
  onToggle: ExtensionsSheetProps['onToggle']
}

function ExtensionRow({ item, onToggle }: ExtensionRowProps): ReactElement {
  return (
    <div className="flex items-center gap-2.5 px-2 py-2">
      <span className="grid size-7 shrink-0 place-items-center rounded-row bg-card text-ink-muted shadow-card">
        <IconPuzzle size={15} />
      </span>
      <div className="min-w-0 flex-1">
        <p className="flex items-baseline gap-2 truncate text-[12.5px] text-ink">
          {item.name}
          <span className="numerique text-[10.5px] text-ink-faint">{item.version}</span>
        </p>
        <p className="truncate text-[11px] text-ink-faint">{item.summary}</p>
      </div>
      <Toggle checked={item.enabled} label={item.name} onChange={(next) => onToggle(item.id, next)} />
    </div>
  )
}

export function ExtensionsSheet({ extensions, onToggle }: ExtensionsSheetProps): ReactElement {
  if (extensions.length === 0) {
    return (
      <EmptyState
        icon={<IconPuzzle size={20} />}
        title="Aucune extension installée"
        hint="Les extensions chargées par le cœur apparaîtront ici, avec un interrupteur chacune."
      />
    )
  }
  return (
    <div className="divide-y divide-hairline">
      {extensions.map((item) => (
        <ExtensionRow key={item.id} item={item} onToggle={onToggle} />
      ))}
    </div>
  )
}
