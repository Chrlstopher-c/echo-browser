// Responsabilite : barre de navigation — deplacements, adresse, bouclier, menu, progression.

import type { ReactElement } from 'react'
import type { ShieldView, TabView } from '../shared/contract'
import { ChromeButton } from '../shared/design/chrome-button'
import { NAV_BAR_HEIGHT } from '../shared/design/geometry'
import { IconBack, IconClose, IconForward, IconMenu, IconReload } from '../shared/design/icons'
import { ShieldButton } from '../shield/shield-button'
import { AddressField } from './address-field'
import { ProgressBar } from './progress-bar'

export interface NavigationBarProps {
  tab: TabView | null
  shield: ShieldView
  shieldOpen: boolean
  menuOpen: boolean
  onBack: () => void
  onForward: () => void
  onReload: () => void
  onStop: () => void
  onNavigate: (input: string) => void
  onShield: () => void
  onMenu: () => void
}

export function NavigationBar(props: NavigationBarProps): ReactElement {
  const { tab, shield, shieldOpen, menuOpen } = props
  const loading = tab?.loading ?? false
  return (
    <div
      style={{ height: NAV_BAR_HEIGHT }}
      className="relative flex shrink-0 items-center gap-1 border-b border-hairline bg-surface px-2"
    >
      <ChromeButton label="Page précédente" onClick={props.onBack} disabled={!(tab?.canGoBack ?? false)}>
        <IconBack />
      </ChromeButton>
      <ChromeButton label="Page suivante" onClick={props.onForward} disabled={!(tab?.canGoForward ?? false)}>
        <IconForward />
      </ChromeButton>
      <ChromeButton
        label={loading ? 'Arrêter le chargement' : 'Recharger la page'}
        onClick={loading ? props.onStop : props.onReload}
        disabled={tab === null}
      >
        {loading ? <IconClose size={15} /> : <IconReload />}
      </ChromeButton>
      <div className="mx-1 flex min-w-0 flex-1">
        <AddressField tab={tab} onSubmit={props.onNavigate} />
      </div>
      <ShieldButton shield={shield} open={shieldOpen} onClick={props.onShield} />
      <ChromeButton label="Menu" onClick={props.onMenu} active={menuOpen}>
        <IconMenu />
      </ChromeButton>
      <ProgressBar loading={loading} progress={tab?.progress ?? 0} />
    </div>
  )
}
