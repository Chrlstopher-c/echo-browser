// Responsabilite : contenu de la barre depliee — controles, adresse, onglets, feuilles, outils, espaces.

import type { ReactElement } from 'react'
import { AddressField } from '../address/address-field'
import { ExtensionBar } from '../extensions/extension-bar'
import { ExtensionMenu } from '../extensions/extension-menu'
import { TopControls } from '../address/top-controls'
import { RestartStrip } from '../restart/restart-strip'
import { SpaceStrip } from '../spaces/space-strip'
import { TabsArea } from '../tabs/tabs-area'
import { NoticeStrip } from './notice-strip'
import { PermissionStrip } from './permission-strip'
import { SidebarSheets } from './sidebar-sheets'
import { SimulationMark } from './simulation-mark'
import { UtilityRow } from './utility-row'
import type { SidebarModel } from './use-sidebar'

function Extensions({ model }: { model: SidebarModel }): ReactElement {
  const { extensions } = model
  return (
    <>
      <ExtensionBar
        extensions={extensions.extensions}
        openId={extensions.popupId}
        onOpen={extensions.openPopup}
        onMenu={model.extensionMenu.openFor}
      />
      <ExtensionMenu
        controller={extensions}
        menu={model.extensionMenu}
        onAskRemove={(id) => {
          extensions.askRemove(id)
          model.sheet.open('extensions')
        }}
      />
    </>
  )
}

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
      <Extensions model={model} />
    </header>
  )
}

function ColumnFooter({ model }: { model: SidebarModel }): ReactElement {
  const { core, sheet, extensions, library, space } = model
  return (
    <footer className="flex shrink-0 flex-col gap-1.5">
      <RestartStrip pending={extensions.restartPending} count={extensions.pending} onRestart={extensions.restart} />
      <PermissionStrip requests={core.state.permissions} send={core.send} />
      <NoticeStrip notice={core.notice} />
      <UtilityRow
        shield={core.shield}
        open={sheet.current}
        compact={false}
        restartPending={extensions.restartPending}
        downloads={library.downloads.summary}
        onToggle={sheet.toggle}
        onOpenTerminal={() => core.send({ kind: 'openTerminal' })}
      />
      <SpaceStrip
        current={space.space.id}
        scheme={space.space.scheme}
        onSelect={space.select}
        onToggleScheme={space.toggleScheme}
      />
    </footer>
  )
}

export function SidebarColumn({ model }: { model: SidebarModel }): ReactElement {
  const { core, tabs, menu, folders, containers, sheet } = model
  return (
    <div className="flex h-full flex-col gap-3 px-3 pt-2 pb-1.5">
      <ColumnHeader model={model} />
      <div className="relative min-h-0 flex-1">
        <div
          onContextMenu={menu.openArea}
          className={`h-full overflow-y-auto ${sheet.current === null ? '' : 'invisible'}`}
        >
          <TabsArea tabs={core.state.tabs} activeId={core.state.activeId} actions={tabs} menu={menu}
            folders={folders} containers={containers} compact={false} />
        </div>
        <SidebarSheets model={model} />
      </div>
      <ColumnFooter model={model} />
    </div>
  )
}
