// Responsabilite : rangee de controles compacts — precedent, suivant, rechargement, repli de la barre.

import type { ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import { IconButton } from '../shared/design/icon-button'
import { IconBack, IconClose, IconForward, IconReload, IconSidebar } from '../shared/design/icons'

export interface TopControlsProps {
  tab: TabView | null
  onBack: () => void
  onForward: () => void
  onReload: () => void
  onStop: () => void
  onCollapse: () => void
  /** Marqueur pose au centre de la rangee (simulation). */
  mark: ReactElement | null
}

export function TopControls(props: TopControlsProps): ReactElement {
  const { tab, mark } = props
  const loading = tab?.loading ?? false
  return (
    <div className="flex items-center gap-0.5">
      <IconButton label="Page précédente" onClick={props.onBack} disabled={!(tab?.canGoBack ?? false)}>
        <IconBack size={15} />
      </IconButton>
      <IconButton label="Page suivante" onClick={props.onForward} disabled={!(tab?.canGoForward ?? false)}>
        <IconForward size={15} />
      </IconButton>
      <IconButton
        label={loading ? 'Arrêter le chargement' : 'Recharger la page'}
        onClick={loading ? props.onStop : props.onReload}
        disabled={tab === null}
      >
        {loading ? <IconClose size={14} /> : <IconReload size={14} />}
      </IconButton>
      <span className="flex flex-1 justify-center">{mark}</span>
      <IconButton label="Replier la barre" onClick={props.onCollapse}>
        <IconSidebar size={15} />
      </IconButton>
    </div>
  )
}
