// Responsabilite : la barre laterale — largeur animee, bascule entre colonne depliee et rail.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { useEssentials } from '../tabs/use-essentials'
import { SidebarColumn } from './sidebar-column'
import { CHROME_EASE, COLLAPSE_SECONDS } from './sidebar-geometry'
import { SidebarRail } from './sidebar-rail'
import type { SidebarModel } from './use-sidebar'

export function Sidebar({ model }: { model: SidebarModel }): ReactElement {
  const essentials = useEssentials(model.core.state.tabs, model.core.simulated)
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
          {collapsed ? (
            <SidebarRail model={model} essentials={essentials} />
          ) : (
            <SidebarColumn model={model} essentials={essentials} />
          )}
        </motion.div>
      </AnimatePresence>
    </motion.aside>
  )
}
