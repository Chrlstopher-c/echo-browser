// Responsabilite : bouton recharger qui devient arreter pendant le chargement — l'icone tourne pour
// se transformer, et un anneau autour dessine l'avancement reel de la page.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import { IconReload, IconStop } from '../shared/design/icons'
import { QUICK } from '../shared/design/motion'
import { ProgressRing } from '../shared/design/progress-ring'

export interface ReloadButtonProps {
  tab: TabView | null
  onReload: () => void
  onStop: () => void
}

function LoadingRing({ tab }: { tab: TabView | null }): ReactElement {
  return (
    <AnimatePresence>
      {tab?.loading === true && (
        <motion.span
          key="anneau"
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 1.2 }}
          transition={QUICK}
          className="absolute inset-0 grid place-items-center"
        >
          <ProgressRing progress={tab.progress} size={22} stroke={1.5} />
        </motion.span>
      )}
    </AnimatePresence>
  )
}

function SwapIcon({ loading }: { loading: boolean }): ReactElement {
  return (
    <AnimatePresence initial={false} mode="popLayout">
      <motion.span
        key={loading ? 'arreter' : 'recharger'}
        initial={{ opacity: 0, rotate: -90, scale: 0.7 }}
        animate={{ opacity: 1, rotate: 0, scale: 1 }}
        exit={{ opacity: 0, rotate: 90, scale: 0.7 }}
        transition={QUICK}
        className="grid place-items-center"
      >
        {loading ? <IconStop size={12} /> : <IconReload size={14} />}
      </motion.span>
    </AnimatePresence>
  )
}

function toneOf(disabled: boolean, loading: boolean): string {
  if (disabled) return 'text-ink-faint/50'
  return loading ? 'text-guard hover:bg-hover' : 'text-ink-muted hover:bg-hover hover:text-ink'
}

export function ReloadButton({ tab, onReload, onStop }: ReloadButtonProps): ReactElement {
  const loading = tab?.loading ?? false
  const disabled = tab === null
  return (
    <button
      type="button"
      disabled={disabled}
      title={loading ? 'Arrêter le chargement (Échap)' : 'Recharger la page (Ctrl+R)'}
      aria-label={loading ? 'Arrêter le chargement' : 'Recharger la page'}
      onClick={loading ? onStop : onReload}
      className={`relative grid size-7 shrink-0 place-items-center rounded-row transition-colors duration-100
        ${toneOf(disabled, loading)}`}
    >
      <LoadingRing tab={tab} />
      <SwapIcon loading={loading} />
    </button>
  )
}
