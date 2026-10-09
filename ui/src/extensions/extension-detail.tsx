// Responsabilite : le detail d'une extension deplie sous sa ligne — ce qu'elle fait, ce qu'elle
// reclame, et les pages qu'elle expose.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { ExtensionView } from '../shared/contract'
import { PushButton } from '../shared/design/push-button'
import { IconOpen, IconSettings } from '../shared/design/icons'
import { QUICK } from '../shared/design/motion'
import { permissionLabel } from './extension-model'
import type { ExtensionsController } from './use-extensions'

/** Au-dela, la liste des permissions noie la ligne : le reste se compte. */
const SHOWN_PERMISSIONS = 8

function Permissions({ permissions }: { permissions: string[] }): ReactElement | null {
  if (permissions.length === 0) return null
  const shown = permissions.slice(0, SHOWN_PERMISSIONS)
  const rest = permissions.length - shown.length
  return (
    <div className="flex flex-wrap gap-1">
      {shown.map((permission) => (
        <span
          key={permission}
          title={permission}
          className="rounded-full bg-hover px-1.5 py-[1px] text-[10.5px] text-ink-muted"
        >
          {permissionLabel(permission)}
        </span>
      ))}
      {rest > 0 && <span className="px-1 py-[1px] text-[10.5px] text-ink-faint">+{rest}</span>}
    </div>
  )
}

export interface ExtensionDetailProps {
  item: ExtensionView
  controller: ExtensionsController
}

export function ExtensionDetail({ item, controller }: ExtensionDetailProps): ReactElement {
  return (
    <motion.div
      initial={{ height: 0, opacity: 0 }}
      animate={{ height: 'auto', opacity: 1 }}
      exit={{ height: 0, opacity: 0 }}
      transition={QUICK}
      className="overflow-hidden"
    >
      <div className="flex flex-col gap-2 px-2 pt-0.5 pb-2.5">
        {item.description !== '' && (
          <p className="text-[11.5px] leading-relaxed text-ink-muted">{item.description}</p>
        )}
        <Permissions permissions={item.permissions} />
        <div className="flex flex-wrap items-center gap-1.5">
          {item.popup !== null && (
            <PushButton onClick={() => controller.openPopupCentered(item.id)} icon={<IconOpen size={11} />}>
              Ouvrir
            </PushButton>
          )}
          {item.options !== null && (
            <PushButton onClick={() => controller.openOptions(item.id)} icon={<IconSettings size={11} />}>
              Réglages
            </PushButton>
          )}
          <span className="numerique ml-auto truncate text-[10.5px] text-ink-faint" title={item.id}>
            {item.id.slice(0, 12)}…
          </span>
        </div>
      </div>
    </motion.div>
  )
}
