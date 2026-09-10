// Responsabilite : la barre laterale — largeur animee, bascule entre colonne depliee et rail.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { CHROME_EASE, COLLAPSE_SECONDS } from '../shared/design/motion'
import { SidebarColumn } from './sidebar-column'
import { SidebarRail } from './sidebar-rail'
import type { SidebarModel } from './use-sidebar'

export function Sidebar({ model }: { model: SidebarModel }): ReactElement {
  const { collapsed, width } = model.width
  return (
    <motion.aside
      initial={false}
      animate={{ width }}
      transition={{ duration: COLLAPSE_SECONDS, ease: CHROME_EASE }}
      onAnimationComplete={model.width.onSettled}
      className="fond-espace relative h-full shrink-0 overflow-hidden"
    >
      <AnimatePresence initial={false} mode="popLayout">
        <motion.div
          key={collapsed ? 'rail' : 'colonne'}
          style={{ width }}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.12 }}
          className="h-full"
        >
          {collapsed ? <SidebarRail model={model} /> : <SidebarColumn model={model} />}
        </motion.div>
      </AnimatePresence>
    </motion.aside>
  )
}
