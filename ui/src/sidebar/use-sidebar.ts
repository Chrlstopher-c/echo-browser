// Responsabilite : modele complet de la barre — coeur, gestes, largeur, feuilles, espace, domaines.

import { useCallback, useEffect, useState } from 'react'
import { useExtensions, type ExtensionsController } from '../extensions/use-extensions'
import { useLibrary, type LibraryController } from '../library/use-library'
import { useSettings, type SettingsController } from '../settings/use-settings'
import type { UiRequest } from '../shared/contract'
import type { CoreState } from '../shared/core-state'
import { useCore, type CoreConnection } from '../shared/use-core'
import { useShield, type ShieldController } from '../shield/use-shield'
import { useSpace, type SpaceController } from '../spaces/use-space'
import { useTabActions, type TabActions } from '../tabs/use-tab-actions'
import { useTabMenu, type TabMenuController } from '../tabs/use-tab-menu'
import { useKeyboard } from './use-keyboard'
import { useSheet, type SheetController } from './use-sheet'
import { useSidebarWidth, type SidebarWidth } from './use-sidebar-width'

export interface SidebarDomains {
  shield: ShieldController
  extensions: ExtensionsController
  library: LibraryController
  settings: SettingsController
}

export interface SidebarModel extends SidebarDomains {
  core: CoreConnection
  tabs: TabActions
  menu: TabMenuController
  width: SidebarWidth
  sheet: SheetController
  space: SpaceController
  /** Incremente a chaque demande de focus du champ d'adresse, d'ou qu'elle vienne. */
  addressFocusToken: number
  /** Deplie la barre si besoin et donne le focus au champ d'adresse. */
  focusAddress: () => void
}

function useDomains(send: (request: UiRequest) => void, state: CoreState): SidebarDomains {
  const shield = useShield(send, {
    lists: state.filterLists, refreshedAt: state.filterListsRefreshedAt, activeId: state.activeId,
  })
  const extensions = useExtensions(send, state.extensions, state.restartPending)
  const library = useLibrary(send, state)
  const settings = useSettings(send, state.settings)
  return { shield, extensions, library, settings }
}

export function useSidebar(): SidebarModel {
  const core = useCore()
  const { state, send } = core
  const tabs = useTabActions(send, state.activeId)
  const menu = useTabMenu()
  const width = useSidebarWidth(send)
  const sheet = useSheet()
  const space = useSpace(send)
  const domains = useDomains(send, state)
  const [localFocus, setLocalFocus] = useState(0)

  const focusAddress = useCallback((): void => {
    width.expand()
    sheet.close()
    setLocalFocus((current) => current + 1)
  }, [width, sheet])

  // Ctrl+L relaye par le coeur : meme chemin que le geste local. Ne reagit qu'au jeton du coeur.
  useEffect(() => {
    if (state.addressFocusToken > 0) focusAddress()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [state.addressFocusToken])

  useKeyboard({ tabs, activeId: state.activeId, space, focusAddress })

  return {
    ...domains, core, tabs, menu, width, sheet, space,
    addressFocusToken: localFocus + state.addressFocusToken,
    focusAddress,
  }
}
