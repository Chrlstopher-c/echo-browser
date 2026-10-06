// Responsabilite : le menu contextuel de la zone des onglets (onglet, dossier, fond de liste).
// Rendu hors de la barre (portail) pour ne pas etre rogne par son defilement.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { createPortal } from 'react-dom'
import type { TabView } from '../shared/contract'
import { QUICK } from '../shared/design/motion'
import { AreaBody, FolderBody, TabBody } from './tab-menu-bodies'
import type { ContainerActions } from './use-containers'
import type { FolderActions } from './use-folders'
import type { TabMenuController, TabMenuState } from './use-tab-menu'
import type { TabActions } from './use-tab-actions'

export interface TabMenuProps {
  tabs: TabView[]
  controller: TabMenuController
  actions: TabActions
  folders: FolderActions
  containers: ContainerActions
  /** L'onglet affiche : on ne l'endort pas. */
  activeId: number | null
}

function keyOf(menu: TabMenuState): string {
  return menu.target.kind === 'area' ? 'area' : `${menu.target.kind}-${menu.target.id}`
}

function Body(props: TabMenuProps & { menu: TabMenuState }): ReactElement | null {
  const { menu, tabs, actions, folders, containers, activeId, controller } = props
  const common = { actions, folders, containers, close: controller.close }
  const { target } = menu
  if (target.kind === 'area') return <AreaBody {...common} />
  if (target.kind === 'folder') {
    return <FolderBody {...common} id={target.id} members={tabs.filter((tab) => tab.folder === target.id)} />
  }
  const tab = tabs.find((candidate) => candidate.id === target.id)
  if (tab === undefined) return null
  const others = tabs.filter((other) => other.id !== tab.id && !other.pinned)
  return <TabBody {...common} tab={tab} activeId={activeId} others={others} />
}

export function TabMenu(props: TabMenuProps): ReactElement {
  const { menu } = props.controller
  return createPortal(
    <AnimatePresence>
      {menu !== null && (
        <motion.div
          key={keyOf(menu)}
          data-tab-menu
          role="menu"
          initial={{ opacity: 0, scale: 0.96, y: -4 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.98 }}
          transition={QUICK}
          style={{ left: menu.x, top: menu.y, width: 224 }}
          className="fixed z-50 origin-top-left rounded-tile bg-card p-1 shadow-lift"
        >
          <Body {...props} menu={menu} />
        </motion.div>
      )}
    </AnimatePresence>,
    document.body,
  )
}
