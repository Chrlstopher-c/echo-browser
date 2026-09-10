// Responsabilite : composition de la fenetre — la barre, et en simulation la scene de la page.

import { MotionConfig } from 'framer-motion'
import type { ReactElement } from 'react'
import { RestartScreen } from './restart/restart-screen'
import { Sidebar } from './sidebar/sidebar'
import { useSidebar } from './sidebar/use-sidebar'
import { PageStage } from './stage/page-stage'

export function App(): ReactElement {
  const model = useSidebar()
  const restarting = model.core.state.restarting
  if (restarting !== null) return <RestartScreen reason={restarting} />
  return (
    <MotionConfig reducedMotion="user">
      <div className="flex h-full overflow-hidden">
        <Sidebar model={model} />
        {model.core.simulated && <PageStage tab={model.core.activeTab} />}
      </div>
    </MotionConfig>
  )
}
