// Responsabilite : assemblage de la bande de chrome — onglets, navigation, panneaux, messages.

import type { ReactElement } from 'react'
import { NavigationBar } from '../chrome/navigation-bar'
import { TabBar } from '../chrome/tab-bar'
import { useCore, type CoreConnection } from '../shared/use-core'
import { ChromePanels } from './chrome-panels'
import { NoticeStrip } from './notice-strip'
import { usePanel, type PanelController } from './use-panel'
import { useChromeActions, type ChromeActions } from './use-chrome-actions'

interface WiringProps {
  core: CoreConnection
  panel: PanelController
  actions: ChromeActions
}

function SimulationMark(): ReactElement {
  return (
    <span
      title="Le cœur Rust est absent : le faux cœur de développement alimente l'interface."
      className="pointer-events-none absolute top-1 right-2 z-20 rounded bg-warn/12 px-1.5 py-px
        text-[9.5px] tracking-[0.08em] text-warn uppercase"
    >
      simulation
    </span>
  )
}

function ChromeBars({ core, panel, actions }: WiringProps): ReactElement {
  return (
    <>
      <TabBar
        tabs={core.state.tabs}
        activeId={core.state.activeId}
        onSelect={actions.selectTab}
        onClose={actions.closeTab}
        onNew={actions.newTab}
      />
      <NavigationBar
        tab={core.activeTab}
        shield={core.shield}
        shieldOpen={panel.open === 'shield'}
        menuOpen={panel.open === 'menu'}
        onBack={actions.back}
        onForward={actions.forward}
        onReload={actions.reload}
        onStop={actions.stop}
        onNavigate={actions.navigate}
        onShield={() => panel.toggle('shield')}
        onMenu={() => panel.toggle('menu')}
      />
    </>
  )
}

export function BrowserChrome(): ReactElement {
  const core = useCore()
  const panel = usePanel(core.send)
  const actions = useChromeActions(core.send, core.state.activeId)
  return (
    <div className="relative flex h-full flex-col overflow-hidden bg-shell">
      <ChromeBars core={core} panel={panel} actions={actions} />
      <ChromePanels
        panel={panel.open}
        shield={core.shield}
        url={core.activeTab?.url ?? ''}
        filterListCount={core.state.filterListCount}
        actions={actions}
        onOpenPanel={panel.show}
      />
      <NoticeStrip notice={core.notice} />
      {core.simulated && <SimulationMark />}
    </div>
  )
}
