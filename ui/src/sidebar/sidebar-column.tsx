// Responsabilite : contenu de la barre depliee — controles, adresse, onglets, feuilles, outils, espaces.

import type { ReactElement } from 'react'
import { AddressField } from '../address/address-field'
import { ExtensionBar } from '../extensions/extension-bar'
import { DownloadStrip } from '../library/downloads/download-strip'
import { FindBar } from '../find/find-bar'
import { ExtensionMenu } from '../extensions/extension-menu'
import { TopControls } from '../address/top-controls'
import { RestartStrip } from '../restart/restart-strip'
import { SpaceStrip } from '../spaces/space-strip'
import { TabsArea } from '../tabs/tabs-area'
import { TabSend } from '../tabs/container-names'
import { CodecsStrip } from './codecs-strip'
import { RoutineStrip } from '../routines/routine-strip'
import { PageChangeStrip } from '../watch/page-change-strip'
import { DiscoverStrip } from '../help/discover-strip'
import { SyncStrip } from './sync-strip'
import { UpdateStrip } from './update-strip'
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
        onOpenSecurity={() => model.sheet.open('network')}
        suggestions={core.state.suggestions}
        send={core.send}
        bookmarked={active === null || !/^(https?|file):/.test(active.url)
          ? null : core.state.bookmarks.some((b) => b.url === active.url)}
        onToggleBookmark={() => {
          if (active === null) return
          const known = core.state.bookmarks.some((b) => b.url === active.url)
          core.send(known ? { kind: 'removeBookmark', url: active.url } : { kind: 'addBookmark', id: active.id })
        }}
      />
      <FindBar find={model.find} result={core.state.find} />
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
      <CodecsStrip codecs={core.state.codecs} send={core.send} />
      <DiscoverStrip send={core.send} ready={/^https?:/.test(core.activeTab?.url ?? '')
        && core.state.settings.some((s) => s.key === 'onboarding.done' && s.value.type === 'flag' && s.value.value)} />
      <PageChangeStrip change={core.state.pageChange} />
      <RoutineStrip proposal={core.state.routineProposal} send={core.send} />
      <SyncStrip account={core.state.account} send={core.send} />
      <UpdateStrip update={core.state.update} send={core.send} />
      <DownloadStrip downloads={library.downloads} />
      <NoticeStrip notice={core.notice} send={core.send} />
      <UtilityRow
        shield={core.shield}
        open={sheet.current}
        compact={false}
        restartPending={extensions.restartPending}
        downloads={library.downloads.summary}
        terminal={core.state.settings.some((s) => s.key === 'system.claudeCode' && s.value.type === 'flag'
          && s.value.value)}
        onToggle={sheet.toggle}
        onOpenTerminal={() => core.send({ kind: 'openTerminal' })}
        onOpenPage={(page) => core.send({ kind: 'openPage', page })}
      />
      <SpaceStrip
        current={space.space.id}
        scheme={space.space.scheme}
        profiles={model.profiles.list}
        onSelect={space.select}
        onCreate={() => space.select(model.profiles.create(''))}
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
          <TabSend.Provider value={core.send}>
            <TabsArea tabs={core.state.tabs.filter((tab) => tab.space === model.space.space.id)}
              activeId={core.state.activeId} actions={tabs} menu={menu}
              folders={folders} containers={containers} compact={false} />
          </TabSend.Provider>
        </div>
        <SidebarSheets model={model} />
      </div>
      <ColumnFooter model={model} />
    </div>
  )
}
