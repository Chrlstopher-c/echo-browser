// Responsabilite : panneau des extensions — liste, interrupteur par extension, avertissement de relance.

import type { ReactElement } from 'react'
import { EmptyState } from '../shared/design/empty-state'
import { IconPuzzle } from '../shared/design/icons'
import { PanelShell } from '../shared/design/panel-shell'
import { Toggle } from '../shared/design/toggle'
import { RESTART_NOTICE, type Extension } from './extension-model'

export interface ExtensionsPanelProps {
  extensions: Extension[]
  onToggle: (id: string, enabled: boolean) => void
}

interface ExtensionRowProps {
  item: Extension
  onToggle: ExtensionsPanelProps['onToggle']
}

function ExtensionRow({ item, onToggle }: ExtensionRowProps): ReactElement {
  return (
    <div className="flex items-center gap-3 px-3.5 py-2.5">
      <span className="grid size-7 shrink-0 place-items-center rounded-md bg-raised text-ink-muted">
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

export function ExtensionsPanel({ extensions, onToggle }: ExtensionsPanelProps): ReactElement {
  return (
    <PanelShell title="Extensions" footer={RESTART_NOTICE}>
      {extensions.length === 0 ? (
        <EmptyState
          icon={<IconPuzzle size={20} />}
          title="Aucune extension installée"
          hint="Les extensions chargées par le cœur apparaîtront ici, avec un interrupteur chacune."
        />
      ) : (
        <div className="divide-y divide-hairline">
          {extensions.map((item) => (
            <ExtensionRow key={item.id} item={item} onToggle={onToggle} />
          ))}
        </div>
      )}
    </PanelShell>
  )
}
