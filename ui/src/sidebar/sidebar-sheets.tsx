// Responsabilite : choix de la feuille affichee sur la liste d'onglets.

import type { ReactElement } from 'react'
import type { ShieldView } from '../shared/contract'
import { ExtensionsSheet } from '../extensions/extensions-sheet'
import type { ExtensionsController } from '../extensions/use-extensions'
import { LibrarySheet } from '../library/library-sheet'
import { useLibrary } from '../library/use-library'
import { SettingsSheet } from '../settings/settings-sheet'
import { FilterListFooter, ShieldSheet } from '../shield/shield-sheet'
import type { SpaceController } from '../spaces/use-space'
import type { SheetId } from './sheet'
import { SheetHost } from './sheet-host'
import type { SidebarActions } from './use-sidebar-actions'
import type { SheetController } from './use-sheet'

export interface SidebarSheetsProps {
  sheet: SheetController
  shield: ShieldView
  url: string
  filterListCount: number | null
  actions: SidebarActions
  space: SpaceController
  extensions: ExtensionsController
}

type LibrarySheetId = Extract<SheetId, 'bookmarks' | 'history' | 'downloads'>

function isLibrary(sheet: SheetId | null): sheet is LibrarySheetId {
  return sheet === 'bookmarks' || sheet === 'history' || sheet === 'downloads'
}

export function SidebarSheets(props: SidebarSheetsProps): ReactElement {
  const { sheet, shield, url, filterListCount, actions, space, extensions } = props
  const library = useLibrary()
  const footer =
    sheet.current === 'shield' ? (
      <FilterListFooter count={filterListCount} onRefresh={actions.refreshLists} />
    ) : undefined
  return (
    <SheetHost sheet={sheet} footer={footer}>
      {sheet.current === 'shield' && (
        <ShieldSheet
          shield={shield}
          url={url}
          onToggleGlobal={actions.setShieldEnabled}
          onToggleSite={actions.toggleShieldSite}
        />
      )}
      {sheet.current === 'extensions' && <ExtensionsSheet controller={extensions} />}
      {sheet.current === 'settings' && (
        <SettingsSheet
          space={space.space.id}
          spaceName={space.space.name}
          onSelectSpace={space.select}
          onOpen={sheet.push}
          onDevTools={actions.devTools}
        />
      )}
      {isLibrary(sheet.current) && <LibrarySheet section={sheet.current} content={library} />}
    </SheetHost>
  )
}
