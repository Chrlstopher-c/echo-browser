// Responsabilite : contenu de la barre repliee — un rail de favicons, sans texte ni feuille.

import type { ReactElement } from 'react'
import { SecurityMark } from '../address/security-mark'
import { IconButton } from '../shared/design/icon-button'
import { IconSidebar } from '../shared/design/icons'
import { TabsArea } from '../tabs/tabs-area'
import type { EssentialsController } from '../tabs/use-essentials'
import { UtilityRow } from './utility-row'
import type { SidebarModel } from './use-sidebar'

export interface SidebarRailProps {
  model: SidebarModel
  essentials: EssentialsController
}

function RailHeader({ model }: { model: SidebarModel }): ReactElement {
  const tab = model.core.activeTab
  const security = tab === null || tab.url.length === 0 ? 'blank' : tab.security
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
        className="grid h-8 w-full place-items-center rounded-row bg-field shadow-card transition-colors
          duration-100 hover:bg-hover"
      >
        <SecurityMark security={security} size={14} />
      </button>
    </header>
  )
}

export function SidebarRail({ model, essentials }: SidebarRailProps): ReactElement {
  const { core, actions, width, sheet } = model
  const openSheet = (id: Parameters<typeof sheet.toggle>[0]): void => {
    width.expand()
    sheet.toggle(id)
  }
  return (
    <div className="flex h-full flex-col items-stretch gap-3 px-3 pt-2 pb-2">
      <RailHeader model={model} />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <TabsArea essentials={essentials} activeId={core.state.activeId} actions={actions} compact />
      </div>
      <footer className="flex justify-center">
        <UtilityRow shield={core.shield} open={sheet.current} compact onToggle={openSheet} />
      </footer>
    </div>
  )
}
