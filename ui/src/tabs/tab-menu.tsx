// Responsabilite : le menu contextuel d'un onglet — epingler, favori, zoom, recharger, fermer.
// Rendu hors de la barre (portail) pour ne pas etre rogne par son defilement.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement, ReactNode } from 'react'
import { createPortal } from 'react-dom'
import type { TabView } from '../shared/contract'
import { formatZoom } from '../shared/format'
import { IconClose, IconPin, IconReload, IconStar, IconUnpin } from '../shared/design/icons'
import { QUICK } from '../shared/design/motion'
import { Stepper } from '../shared/design/stepper'
import type { TabMenuController } from './use-tab-menu'
import type { TabActions } from './use-tab-actions'

export interface TabMenuProps {
  tabs: TabView[]
  controller: TabMenuController
  actions: TabActions
}

function MenuItem({ icon, label, onClick, danger = false }: {
  icon: ReactNode; label: string; onClick: () => void; danger?: boolean
}): ReactElement {
  return (
    <button
      type="button"
      role="menuitem"
      onClick={onClick}
      className={`flex h-7 w-full items-center gap-2.5 rounded-[6px] px-2 text-left text-[12px]
        transition-colors duration-100 hover:bg-hover
        ${danger ? 'text-ink-muted hover:text-danger' : 'text-ink'}`}
    >
      <span className="text-ink-muted">{icon}</span>
      {label}
    </button>
  )
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

function MenuBody({ tab, actions, close }: { tab: TabView; actions: TabActions; close: () => void }): ReactElement {
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
      <div className="my-1 border-t border-hairline" />
      <ZoomRow tab={tab} actions={actions} />
      <div className="my-1 border-t border-hairline" />
      <MenuItem icon={<IconClose size={13} />} label="Fermer l'onglet" danger
        onClick={run(() => actions.close(tab.id))} />
    </>
  )
}

export function TabMenu({ tabs, controller, actions }: TabMenuProps): ReactElement {
  const { menu, close } = controller
  const tab = menu === null ? undefined : tabs.find((candidate) => candidate.id === menu.id)
  return createPortal(
    <AnimatePresence>
      {menu !== null && tab !== undefined && (
        <motion.div
          key={menu.id}
          data-tab-menu
          role="menu"
          initial={{ opacity: 0, scale: 0.96, y: -4 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.98 }}
          transition={QUICK}
          style={{ left: menu.x, top: menu.y, width: 196 }}
          className="fixed z-50 origin-top-left rounded-tile bg-card p-1 shadow-lift"
        >
          <MenuBody tab={tab} actions={actions} close={close} />
        </motion.div>
      )}
    </AnimatePresence>,
    document.body,
  )
}
