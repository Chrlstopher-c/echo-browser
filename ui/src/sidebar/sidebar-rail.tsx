// Responsabilite : contenu de la barre repliee — un rail de favicons, sans texte ni feuille.

import type { ReactElement } from 'react'
import { SecurityMark } from '../address/security-mark'
import { IconButton } from '../shared/design/icon-button'
import { IconSidebar } from '../shared/design/icons'
import { ProgressRing } from '../shared/design/progress-ring'
import { TabsArea } from '../tabs/tabs-area'
import type { SheetId } from './sheet'
import { UtilityRow } from './utility-row'
import type { SidebarModel } from './use-sidebar'

function RailHeader({ model }: { model: SidebarModel }): ReactElement {
  const tab = model.core.activeTab
  const security = tab === null || tab.url.length === 0 || tab.url === 'about:blank' ? 'blank' : tab.security
  return (
    <header className="flex flex-col items-center gap-2">
      <IconButton label="Déplier la barre" onClick={model.width.toggle}>
        <IconSidebar size={15} />
      </IconButton>
      <button
        type="button"
        title={tab === null ? 'Adresse' : tab.url}
        aria-label="Modifier l'adresse"
        onClick={model.focusAddress}
        className="relative grid h-8 w-full place-items-center rounded-row bg-field shadow-card transition-colors
          duration-100 hover:bg-hover"
      >
        {tab?.loading === true ? (
          <span className="text-guard"><ProgressRing progress={tab.progress} size={16} /></span>
        ) : (
          <SecurityMark security={security} size={14} />
        )}
      </button>
    </header>
  )
}

export function SidebarRail({ model }: { model: SidebarModel }): ReactElement {
  const { core, tabs, menu, width, sheet } = model
  const openSheet = (id: SheetId): void => {
    width.expand()
    sheet.open(id)
  }
  return (
    <div className="flex h-full flex-col items-stretch gap-3 px-3 pt-2 pb-2">
      <RailHeader model={model} />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <TabsArea tabs={core.state.tabs} activeId={core.state.activeId} actions={tabs} menu={menu} compact />
      </div>
      <footer className="flex justify-center">
        <UtilityRow
          shield={core.shield}
          open={sheet.current}
          compact
          restartPending={model.extensions.restartPending}
          downloads={model.library.downloads.summary}
          onToggle={openSheet}
        />
      </footer>
    </div>
  )
}
