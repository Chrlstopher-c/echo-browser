// Responsabilite : scene de developpement — la place de la page, dans un cadre flottant aux coins
// arrondis. Le coeur Rust peint ce cadre lui-meme ; ici il n'existe que pour juger la composition.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import type { FakeControls } from '../shared/fake/fake-core'
import { formatPercent, formatZoom } from '../shared/format'
import { QUICK } from '../shared/design/motion'
import { readUrl } from '../shared/url-shape'
import { STAGE_GUTTER } from '../sidebar/sidebar-geometry'
import { DevToolbar } from './dev-toolbar'

function FakeDocument({ tab }: { tab: TabView | null }): ReactElement {
  if (tab === null) {
    return <p className="text-[13px] text-page-ink/50">Aucun onglet ouvert.</p>
  }
  const shape = readUrl(tab.url)
  return (
    <motion.div style={{ scale: tab.zoom }} transition={QUICK}
      className="flex max-w-[52ch] flex-col items-center gap-2 text-center">
      <p className="text-[22px] font-medium tracking-[-0.02em] text-page-ink">{shape.host || tab.title}</p>
      <p className="numerique text-[12px] text-page-ink/55">{tab.url}</p>
      <p className="mt-4 text-[12px] text-page-ink/45">
        {tab.asleep ? 'Onglet endormi : la page se rechargera au réveil.'
          : tab.loading ? `Chargement… ${formatPercent(tab.progress)}` : 'Ici, Chromium affiche la page.'}
      </p>
      {Math.abs(tab.zoom - 1) > 0.001 && (
        <p className="numerique text-[11px] text-page-ink/40">zoom {formatZoom(tab.zoom)}</p>
      )}
    </motion.div>
  )
}

function FullscreenVideo({ onExit }: { onExit: () => void }): ReactElement {
  return (
    <motion.div key="plein-ecran" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
      transition={QUICK} className="fixed inset-0 z-40 grid place-items-center bg-black text-white/60">
      <div className="flex flex-col items-center gap-3 text-center">
        <p className="text-[15px]">Vidéo en plein écran</p>
        <p className="text-[12px] text-white/40">L’interface s’est effacée. Échap pour en sortir.</p>
        <button type="button" onClick={onExit}
          className="mt-2 rounded-md bg-white/10 px-3 py-1.5 text-[12px] text-white hover:bg-white/20">
          Quitter le plein écran
        </button>
      </div>
    </motion.div>
  )
}

export interface PageStageProps {
  tab: TabView | null
  fake: FakeControls | null
  fullscreen: boolean
  onExitFullscreen: () => void
}

export function PageStage({ tab, fake, fullscreen, onExitFullscreen }: PageStageProps): ReactElement {
  return (
    <main style={{ padding: STAGE_GUTTER, paddingLeft: 0 }}
      className="relative flex min-w-0 flex-1 transition-colors duration-300">
      <div className="grid flex-1 place-items-center overflow-hidden rounded-frame bg-page shadow-frame">
        <FakeDocument tab={tab} />
      </div>
      {fake !== null && <DevToolbar fake={fake} tab={tab} />}
      <AnimatePresence>{fullscreen && <FullscreenVideo onExit={onExitFullscreen} />}</AnimatePresence>
    </main>
  )
}
