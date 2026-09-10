// Responsabilite : une extension dans la liste — etat, interrupteur, retrait avec confirmation.

import type { ReactElement } from 'react'
import type { ExtensionView } from '../shared/contract'
import { IconPuzzle, IconTrash } from '../shared/design/icons'
import { Toggle } from '../shared/design/toggle'
import { extensionStatus } from './extension-model'
import type { ExtensionsController } from './use-extensions'

export interface ExtensionRowProps {
  item: ExtensionView
  controller: ExtensionsController
}

function PendingMark(): ReactElement {
  return (
    <span
      title="Prend effet à la relance"
      className="shrink-0 rounded-full bg-warn/15 px-1.5 py-[1px] text-[10px] font-medium text-warn"
    >
      relance
    </span>
  )
}

function RemoveConfirm({ item, controller }: ExtensionRowProps): ReactElement {
  return (
    <div className="flex items-center gap-2 bg-hover px-2 py-2">
      <p className="min-w-0 flex-1 text-[11.5px] leading-snug text-ink-muted">
        Retirer <span className="text-ink">{item.name}</span> ? Ses données locales sont perdues.
      </p>
      <button
        type="button"
        onClick={controller.cancelRemove}
        className="shrink-0 rounded-md px-2 py-[3px] text-[11px] text-ink-muted hover:text-ink"
      >
        Annuler
      </button>
      <button
        type="button"
        onClick={() => controller.confirmRemove(item.id)}
        className="shrink-0 rounded-md bg-danger/15 px-2 py-[3px] text-[11px] font-medium text-danger
          transition-colors duration-100 hover:bg-danger/25"
      >
        Retirer
      </button>
    </div>
  )
}

export function ExtensionRow({ item, controller }: ExtensionRowProps): ReactElement {
  if (controller.confirming === item.id) return <RemoveConfirm item={item} controller={controller} />
  return (
    <div className="group flex items-center gap-2.5 px-2 py-2">
      <span className="grid size-7 shrink-0 place-items-center rounded-row bg-card text-ink-muted shadow-card">
        <IconPuzzle size={15} />
      </span>
      <div className="min-w-0 flex-1">
        <p className="flex items-baseline gap-2 truncate text-[12.5px] text-ink">
          {item.name}
          <span className="numerique text-[10.5px] text-ink-faint">{item.version}</span>
        </p>
        <p className={`truncate text-[11px] ${item.pending ? 'text-warn' : 'text-ink-faint'}`}>
          {extensionStatus(item)}
        </p>
      </div>
      {item.pending && <PendingMark />}
      <button
        type="button"
        aria-label={`Retirer ${item.name}`}
        title={`Retirer ${item.name}`}
        onClick={() => controller.askRemove(item.id)}
        className="grid size-6 shrink-0 place-items-center rounded-row text-ink-faint opacity-0 transition
          duration-100 group-hover:opacity-100 hover:bg-hover hover:text-danger focus-visible:opacity-100"
      >
        <IconTrash size={13} />
      </button>
      <Toggle
        checked={item.enabled}
        label={`Activer ${item.name}`}
        onChange={(next) => controller.setEnabled(item.id, next)}
      />
    </div>
  )
}
