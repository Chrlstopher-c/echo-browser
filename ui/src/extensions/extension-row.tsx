// Responsabilite : une extension dans la liste — icone, etat, interrupteur, detail depliable,
// retrait confirme en place.

import { AnimatePresence } from 'framer-motion'
import { useState, type ReactElement } from 'react'
import type { ExtensionView } from '../shared/contract'
import { ConfirmStrip } from '../shared/design/confirm-strip'
import { IconPuzzle, IconTrash } from '../shared/design/icons'
import { RowAction } from '../shared/design/list-row'
import { Toggle } from '../shared/design/toggle'
import { ExtensionDetail } from './extension-detail'
import { extensionStatus } from './extension-model'
import type { ExtensionsController } from './use-extensions'

export interface ExtensionRowProps {
  item: ExtensionView
  controller: ExtensionsController
}

function ExtensionIcon({ item }: { item: ExtensionView }): ReactElement {
  return (
    <span className={`grid size-7 shrink-0 place-items-center overflow-hidden rounded-row bg-card text-ink-muted
      shadow-card ${item.enabled ? '' : 'opacity-50 grayscale'}`}>
      {item.icon !== null ? <img src={item.icon} alt="" width={18} height={18} draggable={false} /> :
        <IconPuzzle size={15} />}
    </span>
  )
}

function PendingMark(): ReactElement {
  return (
    <span title="Prend effet à la relance"
      className="shrink-0 rounded-full bg-warn/15 px-1.5 py-[1px] text-[10px] font-medium text-warn">
      relance
    </span>
  )
}

function RemoveConfirm({ item, controller }: ExtensionRowProps): ReactElement {
  return (
    <div className="py-1">
      <ConfirmStrip
        question={<>Retirer <span className="text-ink">{item.name}</span> ? Ses données locales sont perdues.</>}
        confirmLabel="Retirer"
        onConfirm={() => controller.confirmRemove(item.id)}
        onCancel={controller.cancelRemove}
      />
    </div>
  )
}

function Remove({ item, controller }: ExtensionRowProps): ReactElement {
  return (
    <RowAction label={`Retirer ${item.name}`} onClick={() => controller.askRemove(item.id)} danger>
      <IconTrash size={13} />
    </RowAction>
  )
}

export function ExtensionRow({ item, controller }: ExtensionRowProps): ReactElement {
  const [open, setOpen] = useState(false)
  if (controller.confirming === item.id) return <RemoveConfirm item={item} controller={controller} />
  return (
    <div className="group">
      <div className="flex h-11 items-center gap-2.5 px-2">
        <button
          type="button"
          aria-expanded={open}
          aria-label={`Détail de ${item.name}`}
          onClick={() => setOpen((current) => !current)}
          className="flex min-w-0 flex-1 items-center gap-2.5 text-left"
        >
          <ExtensionIcon item={item} />
          <span className="min-w-0 flex-1">
            <span className="flex items-baseline gap-2 truncate text-[12.5px] text-ink">
              <span className="truncate">{item.name}</span>
              <span className="numerique shrink-0 text-[10.5px] text-ink-faint">{item.version}</span>
            </span>
            <span className={`block truncate text-[11px] ${item.pending ? 'text-warn' : 'text-ink-faint'}`}>
              {extensionStatus(item)}
            </span>
          </span>
        </button>
        {item.pending && <PendingMark />}
        <Remove item={item} controller={controller} />
        <Toggle checked={item.enabled} label={`Activer ${item.name}`}
          onChange={(next) => controller.setEnabled(item.id, next)} />
      </div>
      <AnimatePresence initial={false}>
        {open && <ExtensionDetail item={item} controller={controller} />}
      </AnimatePresence>
    </div>
  )
}
