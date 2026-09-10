// Responsabilite : leviers de simulation poses sur la scene — son, sommeil, telechargement, plein ecran.
// N'existe qu'avec le faux coeur, pour juger chaque etat de la barre sans le vrai navigateur.

import type { ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import type { FakeControls } from '../shared/fake/fake-core'
import { IconDownload, IconExitFullscreen, IconMoon, IconVolume } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'

export interface DevToolbarProps {
  fake: FakeControls
  tab: TabView | null
}

export function DevToolbar({ fake, tab }: DevToolbarProps): ReactElement {
  const id = tab?.id ?? null
  return (
    <div className="absolute right-3 bottom-3 flex items-center gap-1.5 rounded-tile bg-shell/80 p-1.5 shadow-lift
      backdrop-blur">
      <span className="px-1.5 text-[9.5px] tracking-[0.08em] text-ink-faint uppercase">simuler</span>
      <PushButton disabled={id === null} onClick={() => id !== null && fake.toggleAudible(id)}
        icon={<IconVolume size={11} />}>
        {tab?.audible === true ? 'Couper le son' : 'Son'}
      </PushButton>
      <PushButton disabled={id === null} onClick={() => id !== null && fake.toggleAsleep(id)}
        icon={<IconMoon size={11} />}>
        {tab?.asleep === true ? 'Réveiller' : 'Endormir'}
      </PushButton>
      <PushButton onClick={fake.startDownload} icon={<IconDownload size={11} />}>Télécharger</PushButton>
      <PushButton onClick={fake.enterFullscreen} icon={<IconExitFullscreen size={11} />}>Plein écran</PushButton>
    </div>
  )
}
