// Responsabilite : la marque d'un onglet — favicon au repos, anneau de progression pendant le
// chargement, favicon eteint et lune quand il dort. Lisible sans texte.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import { IconMoon } from '../shared/design/icons'
import { QUICK } from '../shared/design/motion'
import { ProgressRing } from '../shared/design/progress-ring'
import { SiteMark } from '../shared/design/site-mark'
import { containerColor } from './use-containers'

export interface TabMarkProps {
  tab: TabView
  /** Cote de la marque, en pixels. */
  size: number
}

function SleepBadge({ size }: { size: number }): ReactElement {
  return (
    <motion.span
      initial={{ opacity: 0, scale: 0.6 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.6 }}
      transition={QUICK}
      style={{ width: size, height: size }}
      className="absolute -right-1 -bottom-1 grid place-items-center rounded-full bg-shell text-ink-faint"
    >
      <IconMoon size={size - 2} />
    </motion.span>
  )
}

function ContainerPip({ id }: { id: string }): ReactElement {
  return (
    <span
      title="Conteneur"
      style={{ background: containerColor(id) }}
      className="absolute -top-0.5 -right-0.5 block size-[7px] rounded-full ring-2 ring-shell"
    />
  )
}

export function TabMark({ tab, size }: TabMarkProps): ReactElement {
  const ringSize = size + 6
  return (
    <span style={{ width: size, height: size }} className="relative grid shrink-0 place-items-center">
      <motion.span
        animate={{ opacity: tab.asleep ? 0.38 : tab.loading ? 0.55 : 1, scale: tab.loading ? 0.82 : 1 }}
        transition={QUICK}
        className={`grid place-items-center ${tab.asleep ? 'grayscale' : ''}`}
      >
        <SiteMark url={tab.url} favicon={tab.favicon} size={size} />
      </motion.span>
      {tab.container !== null && <ContainerPip id={tab.container} />}
      <AnimatePresence>
        {tab.loading && (
          <motion.span
            key="anneau"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0, scale: 1.25 }}
            transition={QUICK}
            // Anneau plus grand que l'icone : un centrage par grille le decale (debordement aligne a gauche).
            // On le pose donc en retrait negatif egal de chaque cote.
            style={{ inset: -(ringSize - size) / 2 }}
            className="absolute text-guard"
          >
            <ProgressRing progress={tab.progress} size={ringSize} stroke={1.5} className="block" />
          </motion.span>
        )}
        {tab.asleep && <SleepBadge key="lune" size={Math.round(size * 0.62)} />}
      </AnimatePresence>
    </span>
  )
}

/** Trois barres qui respirent : la page joue du son. */
export function AudioBars({ className = '' }: { className?: string }): ReactElement {
  return (
    <span aria-label="Joue du son" title="Joue du son"
      className={`flex h-3 items-end gap-[2px] ${className}`}>
      {[0, 0.3, 0.15].map((delay, index) => (
        <span key={index} style={{ animationDelay: `${delay}s` }}
          className="barre-son block h-full w-[2px] rounded-full bg-current" />
      ))}
    </span>
  )
}
