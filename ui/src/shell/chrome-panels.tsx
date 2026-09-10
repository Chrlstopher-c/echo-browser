// Responsabilite : choix du panneau affiche sous la barre de navigation.

import type { ReactElement } from 'react'
import type { ShieldView } from '../shared/contract'
import { ExtensionsPanel } from '../extensions/extensions-panel'
import { useExtensions } from '../extensions/use-extensions'
import { LibraryPanel } from '../library/library-panel'
import { useLibrary } from '../library/use-library'
import { ShieldPanel } from '../shield/shield-panel'
import { MenuPanel } from './menu-panel'
import { PanelHost } from './panel-host'
import type { PanelId } from './panel'
import type { ChromeActions } from './use-chrome-actions'

export interface ChromePanelsProps {
  panel: PanelId | null
  shield: ShieldView
  url: string
  filterListCount: number | null
  actions: ChromeActions
  onOpenPanel: (panel: PanelId) => void
}

export function ChromePanels(props: ChromePanelsProps): ReactElement {
  const library = useLibrary()
  const extensions = useExtensions()
  return (
    <PanelHost panel={props.panel}>
      {props.panel === 'shield' && (
        <ShieldPanel
          shield={props.shield}
          url={props.url}
          filterListCount={props.filterListCount}
          onToggleGlobal={props.actions.setShieldEnabled}
          onToggleSite={props.actions.toggleShieldSite}
          onRefreshLists={props.actions.refreshLists}
        />
      )}
      {props.panel === 'menu' && <MenuPanel onOpen={props.onOpenPanel} onDevTools={props.actions.devTools} />}
      {props.panel === 'extensions' && (
        <ExtensionsPanel extensions={extensions.extensions} onToggle={extensions.toggle} />
      )}
      {isLibrary(props.panel) && <LibraryPanel section={props.panel} content={library} />}
    </PanelHost>
  )
}

type LibrarySectionPanel = Extract<PanelId, 'bookmarks' | 'history' | 'downloads'>

function isLibrary(panel: PanelId | null): panel is LibrarySectionPanel {
  return panel === 'bookmarks' || panel === 'history' || panel === 'downloads'
}
