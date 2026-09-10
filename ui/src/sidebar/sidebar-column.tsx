// Responsabilite : contenu de la barre depliee — controles, adresse, onglets, feuilles, outils, espaces.

import type { ReactElement } from 'react'
import { AddressField } from '../address/address-field'
import { TopControls } from '../address/top-controls'
import { RestartStrip } from '../restart/restart-strip'
import { SpaceStrip } from '../spaces/space-strip'
import { TabsArea } from '../tabs/tabs-area'
import { NoticeStrip } from './notice-strip'
import { SidebarSheets } from './sidebar-sheets'
import { SimulationMark } from './simulation-mark'
import { UtilityRow } from './utility-row'
import type { SidebarModel } from './use-sidebar'

function ColumnHeader({ model }: { model: SidebarModel }): ReactElement {
  const { core, tabs, width } = model
  const active = core.activeTab
  return (
    <header className="flex shrink-0 flex-col gap-2">
      <TopControls
        tab={active}
        onBack={tabs.back}
        onForward={tabs.forward}
        onReload={tabs.reloadActive}
        onStop={tabs.stop}
        onCollapse={width.toggle}
        mark={core.simulated ? <SimulationMark /> : null}
      />
      <AddressField
        tab={active}
        onSubmit={tabs.navigate}
        onResetZoom={() => {
          if (active !== null) tabs.setZoom(active.id, 1)
        }}
        focusToken={model.addressFocusToken}
      />
    </header>
  )
}

function ColumnFooter({ model }: { model: SidebarModel }): ReactElement {
  const { core, sheet, extensions, library, space } = model
  return (
    <footer className="flex shrink-0 flex-col gap-1.5">
      <RestartStrip pending={extensions.restartPending} count={extensions.pending} onRestart={extensions.restart} />
      <NoticeStrip notice={core.notice} />
      <UtilityRow
        shield={core.shield}
        open={sheet.current}
        compact={false}
        restartPending={extensions.restartPending}
        downloads={library.downloads.summary}
        onToggle={sheet.toggle}
      />
      <SpaceStrip current={space.space.id} onSelect={space.select} />
    </footer>
  )
}

export function SidebarColumn({ model }: { model: SidebarModel }): ReactElement {
  const { core, tabs, menu, sheet } = model
  return (
    <div className="flex h-full flex-col gap-3 px-3 pt-2 pb-1.5">
      <ColumnHeader model={model} />
      <div className="relative min-h-0 flex-1">
        <div className={`h-full overflow-y-auto ${sheet.current === null ? '' : 'invisible'}`}>
          <TabsArea tabs={core.state.tabs} activeId={core.state.activeId} actions={tabs} menu={menu} compact={false} />
        </div>
        <SidebarSheets model={model} />
      </div>
      <ColumnFooter model={model} />
    </div>
  )
}
