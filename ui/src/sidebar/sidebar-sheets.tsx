// Responsabilite : choix de la feuille affichee sur la liste d'onglets.

import type { ReactElement } from 'react'
import { ExtensionsSheet } from '../extensions/extensions-sheet'
import { LibrarySheet } from '../library/library-sheet'
import { SettingsSheet } from '../settings/settings-sheet'
import { ShieldSheet } from '../shield/shield-sheet'
import { SheetHost } from './sheet-host'
import type { SidebarModel } from './use-sidebar'

function SheetBody({ model }: { model: SidebarModel }): ReactElement | null {
  const { sheet, core } = model
  switch (sheet.current) {
    case 'shield':
      return <ShieldSheet view={core.shield} url={core.activeTab?.url ?? ''} shield={model.shield} />
    case 'library':
      return <LibrarySheet controller={model.library} />
    case 'extensions':
      return <ExtensionsSheet controller={model.extensions} />
    case 'settings':
      return <SettingsSheet settings={model.settings} space={model.space} containers={model.containers}
        grants={core.state.grants}
        onForgetGrant={(origin, permission) => core.send({ kind: 'forgetPermission', origin, permission })}
        onDevTools={model.tabs.devTools} />
    case null:
      return null
  }
}

export function SidebarSheets({ model }: { model: SidebarModel }): ReactElement {
  return (
    <SheetHost sheet={model.sheet}>
      <SheetBody model={model} />
    </SheetHost>
  )
}
