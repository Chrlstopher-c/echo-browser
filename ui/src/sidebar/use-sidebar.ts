// Responsabilite : modele complet de la barre — coeur, gestes, largeur, feuilles, espace, focus adresse.

import { useCallback, useState } from 'react'
import { useExtensions, type ExtensionsController } from '../extensions/use-extensions'
import { useCore, type CoreConnection } from '../shared/use-core'
import { useSpace, type SpaceController } from '../spaces/use-space'
import { useSheet, type SheetController } from './use-sheet'
import { useSidebarActions, type SidebarActions } from './use-sidebar-actions'
import { useSidebarWidth, type SidebarWidth } from './use-sidebar-width'

export interface SidebarModel {
  core: CoreConnection
  actions: SidebarActions
  width: SidebarWidth
  sheet: SheetController
  space: SpaceController
  extensions: ExtensionsController
  /** Incremente a chaque demande de focus du champ d'adresse. */
  addressFocusToken: number
  /** Deplie la barre si besoin et donne le focus au champ d'adresse. */
  focusAddress: () => void
}

export function useSidebar(): SidebarModel {
  const core = useCore()
  const actions = useSidebarActions(core.send, core.state.activeId)
  const width = useSidebarWidth(core.send)
  const sheet = useSheet()
  const space = useSpace(core.send)
  const extensions = useExtensions(core.send, core.state.extensions, core.state.restartPending)
  const [addressFocusToken, setToken] = useState(0)

  const focusAddress = useCallback((): void => {
    width.expand()
    setToken((current) => current + 1)
  }, [width])

  return { core, actions, width, sheet, space, extensions, addressFocusToken, focusAddress }
}
