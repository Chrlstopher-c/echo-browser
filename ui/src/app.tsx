// Responsabilite : composition de la fenetre — la barre, l'ecran de relance, l'effacement en plein
// ecran, et en simulation la scene de la page.

import { AnimatePresence, motion, MotionConfig } from 'framer-motion'
import type { ReactElement } from 'react'
import { RestartScreen } from './restart/restart-screen'
import { QUICK } from './shared/design/motion'
import { Sidebar } from './sidebar/sidebar'
import { useSidebar } from './sidebar/use-sidebar'
import { PageStage } from './stage/page-stage'

export function App(): ReactElement {
  const model = useSidebar()
  const { core } = model
  const { restarting, fullscreen } = core.state
  if (restarting !== null) return <RestartScreen reason={restarting} />
  return (
    <MotionConfig reducedMotion="user">
      <div className="flex h-full overflow-hidden">
        <AnimatePresence initial={false}>
          {!fullscreen && (
            <motion.div key="barre" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
              transition={QUICK} className="flex h-full shrink-0">
              <Sidebar model={model} />
            </motion.div>
          )}
        </AnimatePresence>
        {core.simulated && (
          <PageStage
            tab={core.activeTab}
            fake={core.fake}
            fullscreen={fullscreen}
            onExitFullscreen={() => core.send({ kind: 'exitFullscreen' })}
          />
        )}
      </div>
    </MotionConfig>
  )
}
