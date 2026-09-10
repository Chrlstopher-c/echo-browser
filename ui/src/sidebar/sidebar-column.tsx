// Responsabilite : contenu de la barre depliee — controles, adresse, essentiels, onglets, outils.

import type { ReactElement } from 'react'
import { AddressField } from '../address/address-field'
import { TopControls } from '../address/top-controls'
import { TabsArea } from '../tabs/tabs-area'
import type { EssentialsController } from '../tabs/use-essentials'
import { NoticeStrip } from './notice-strip'
import { SidebarSheets } from './sidebar-sheets'
import { SimulationMark } from './simulation-mark'
import { UtilityRow } from './utility-row'
import type { SidebarModel } from './use-sidebar'

export interface SidebarColumnProps {
  model: SidebarModel
  essentials: EssentialsController
}

function ColumnHeader({ model }: { model: SidebarModel }): ReactElement {
  const { core, actions, width } = model
  return (
    <header className="flex shrink-0 flex-col gap-2">
      <TopControls
        tab={core.activeTab}
        onBack={actions.back}
        onForward={actions.forward}
        onReload={actions.reload}
        onStop={actions.stop}
        onCollapse={width.toggle}
        mark={core.simulated ? <SimulationMark /> : null}
      />
      <AddressField tab={core.activeTab} onSubmit={actions.navigate} focusToken={model.addressFocusToken} />
    </header>
  )
}

export function SidebarColumn({ model, essentials }: SidebarColumnProps): ReactElement {
  const { core, actions, sheet, space } = model
  return (
    <div className="flex h-full flex-col gap-3 px-3 pt-2 pb-2">
      <ColumnHeader model={model} />
      <div className="relative min-h-0 flex-1">
        <div className={`h-full overflow-y-auto ${sheet.current === null ? '' : 'invisible'}`}>
          <TabsArea essentials={essentials} activeId={core.state.activeId} actions={actions} compact={false} />
        </div>
        <SidebarSheets
          sheet={sheet}
          shield={core.shield}
          url={core.activeTab?.url ?? ''}
          filterListCount={core.state.filterListCount}
          actions={actions}
          space={space}
        />
      </div>
      <footer className="shrink-0">
        <NoticeStrip notice={core.notice} />
        <UtilityRow shield={core.shield} open={sheet.current} compact={false} onToggle={sheet.toggle} />
      </footer>
    </div>
  )
}
