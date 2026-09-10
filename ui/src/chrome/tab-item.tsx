// Responsabilite : un onglet de la bande — favicon, titre tronque, fermeture.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import { IconClose, IconGlobe } from '../shared/design/icons'
import { fallbackTitle } from '../shared/url-shape'

export interface TabItemProps {
  tab: TabView
  active: boolean
  onSelect: () => void
  onClose: () => void
}

function TabMark({ tab }: { tab: TabView }): ReactElement {
  if (tab.loading) {
    return (
      <span className="size-3.5 shrink-0 rounded-full border-[1.5px] border-edge border-t-ink-muted
        motion-safe:animate-spin" />
    )
  }
  if (tab.favicon !== null) {
    return <img src={tab.favicon} alt="" className="size-3.5 shrink-0 rounded-[3px]" />
  }
  return <IconGlobe size={14} className="shrink-0 text-ink-faint" />
}

export function TabItem({ tab, active, onSelect, onClose }: TabItemProps): ReactElement {
  const surface = active ? 'bg-surface text-ink' : 'text-ink-muted hover:bg-hover/60 hover:text-ink'
  return (
    <motion.div
      layout="position"
      initial={{ opacity: 0, y: -3 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.12, ease: [0.22, 0.61, 0.36, 1] }}
      onPointerDown={onSelect}
      title={tab.title.length > 0 ? tab.title : fallbackTitle(tab.url)}
      className={`group flex h-[26px] min-w-[58px] max-w-[190px] flex-1 items-center gap-2 rounded-tab
        px-2.5 ${surface}`}
    >
      <TabMark tab={tab} />
      <span className="min-w-0 flex-1 truncate text-[12px]">
        {tab.title.length > 0 ? tab.title : fallbackTitle(tab.url)}
      </span>
      <button
        type="button"
        aria-label="Fermer l'onglet"
        onPointerDown={(event) => event.stopPropagation()}
        onClick={onClose}
        className="grid size-4 shrink-0 place-items-center rounded text-ink-faint opacity-0
          transition-opacity duration-100 group-hover:opacity-100 hover:bg-edge hover:text-ink
          focus-visible:opacity-100"
      >
        <IconClose size={11} />
      </button>
    </motion.div>
  )
}
