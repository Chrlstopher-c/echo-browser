// Responsabilite : modele complet de la barre — coeur, gestes, largeur, feuilles, espace, domaines.

import { useCallback, useEffect, useState } from 'react'
import type { SheetId } from './sheet'
import { useFind, type FindController } from '../find/use-find'
import { useExtensionMenu, type ExtensionMenuController } from '../extensions/extension-menu'
import { useExtensions, type ExtensionsController } from '../extensions/use-extensions'
import { useLibrary, type LibraryController } from '../library/use-library'
import { useSettings, type SettingsController } from '../settings/use-settings'
import type { UiRequest } from '../shared/contract'
import type { CoreState } from '../shared/core-state'
import { useCore, type CoreConnection } from '../shared/use-core'
import { useShield, type ShieldController } from '../shield/use-shield'
import { useSpace, type SpaceController } from '../spaces/use-space'
import { useTabActions, type TabActions } from '../tabs/use-tab-actions'
import { useContainers, type ContainerActions } from '../tabs/use-containers'
import { useProfileNames, type ProfileNames } from '../spaces/use-profile-names'
import { useFolders, type FolderActions } from '../tabs/use-folders'
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
  folders: FolderActions
  containers: ContainerActions
  profiles: ProfileNames
  /** Menu de gestion d'une extension, ouvert au clic droit sur son icone. */
  extensionMenu: ExtensionMenuController
  width: SidebarWidth
  sheet: SheetController
  space: SpaceController
  /** Incremente a chaque demande de focus du champ d'adresse, d'ou qu'elle vienne. */
  addressFocusToken: number
  /** Deplie la barre si besoin et donne le focus au champ d'adresse. */
  focusAddress: () => void
  /** Recherche dans la page (Ctrl+F). */
  find: FindController
}

function useDomains(send: (request: UiRequest) => void, state: CoreState): SidebarDomains {
  const shield = useShield(send, {
    lists: state.filterLists, refreshedAt: state.filterListsRefreshedAt, activeId: state.activeId,
  })
  const extensions = useExtensions(send, state.extensions, state.restartPending, state.extensionPopupId)
  const library = useLibrary(send, state)
  const settings = useSettings(send, state.settings)
  return { shield, extensions, library, settings }
}

export function useSidebar(): SidebarModel {
  const core = useCore()
  const { state, send } = core
  const tabs = useTabActions(send, state.activeId)
  const menu = useTabMenu()
  const folders = useFolders(send, state.settings)
  const containers = useContainers(send, state.settings)
  const profiles = useProfileNames(send, state.settings)
  const extensionMenu = useExtensionMenu()
  const width = useSidebarWidth(send)
  const sheet = useSheet()
  const space = useSpace(send, profiles, state.systemDark)
  const domains = useDomains(send, state)
  const [localFocus, setLocalFocus] = useState(0)
  const find = useFind(send, state.findToken, state.activeId)

  // Ctrl+F : la barre se deplie et quitte un panneau ouvert pour montrer la recherche.
  useEffect(() => {
    if (state.findToken === 0) return
    width.expand()
    sheet.close()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [state.findToken])

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

  // L'Aide (page a part) demande un panneau de la barre : on l'ouvre.
  const requested = state.sheetRequest
  useEffect(() => {
    if (requested === null) return
    const known: SheetId[] = ['network', 'shield', 'extensions']
    const target = known.find((id) => id === requested.sheet)
    if (target !== undefined) {
      width.expand()
      sheet.open(target)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [requested?.token])

  useKeyboard({ tabs, activeId: state.activeId, space, focusAddress })

  return {
    ...domains, core, tabs, menu, folders, containers, profiles, extensionMenu, width, sheet, space,
    addressFocusToken: localFocus + state.addressFocusToken,
    focusAddress,
    find,
  }
}
