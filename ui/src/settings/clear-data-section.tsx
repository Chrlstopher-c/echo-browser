// Responsabilite : « Effacer les données de navigation » du profil affiche — periode, historique, cookies, cache —
// comme Chrome et Firefox (Ctrl+Maj+Suppr y mene).

import { useState, type ReactElement } from 'react'
import type { UiRequest } from '../shared/contract'
import { PushButton } from '../shared/design/push-button'
import { SectionLabel } from '../shared/design/section-label'
import { Segmented, type Segment } from '../shared/design/segmented'
import { Toggle } from '../shared/design/toggle'

type Period = 'heure' | 'jour' | 'semaine' | 'tout'

const PERIODS: Array<Segment<Period>> = [
  { id: 'heure', label: 'Dernière heure' }, { id: 'jour', label: '24 h' },
  { id: 'semaine', label: '7 jours' }, { id: 'tout', label: 'Tout' },
]

const SECONDS: Record<Period, number> = { heure: 3600, jour: 86_400, semaine: 7 * 86_400, tout: 0 }

function since(period: Period): number {
  const span = SECONDS[period]
  return span === 0 ? 0 : Math.floor(Date.now() / 1000) - span
}

function Choice(props: { label: string; detail: string; on: boolean; set: (next: boolean) => void }): ReactElement {
  return (
    <div className="flex items-center justify-between gap-3 px-2 py-1.5">
      <div className="min-w-0">
        <p className="text-[12.5px] text-ink">{props.label}</p>
        <p className="text-[11px] leading-snug text-ink-faint">{props.detail}</p>
      </div>
      <Toggle checked={props.on} label={props.label} onChange={props.set} />
    </div>
  )
}

export function ClearDataSection({ send }: { send: (request: UiRequest) => void }): ReactElement {
  const [period, setPeriod] = useState<Period>('heure')
  const [history, setHistory] = useState(true)
  const [cookies, setCookies] = useState(false)
  const [cache, setCache] = useState(true)
  const nothing = !history && !cookies && !cache
  return (
    <section aria-label="Effacer les données de navigation">
      <SectionLabel>Effacer les données de navigation</SectionLabel>
      <p className="px-2 pb-2 text-[11px] leading-snug text-ink-faint">
        Pour le profil affiché seulement. Ctrl+Maj+Suppr ouvre directement ce panneau.
      </p>
      <div className="px-2 pb-1">
        <Segmented name="effacer-periode" segments={PERIODS} value={period} onChange={setPeriod} />
      </div>
      <Choice label="Historique" detail="Les pages visitées sur la période." on={history} set={setHistory} />
      <Choice label="Cookies et sessions" detail="Vous serez déconnecté des sites de ce profil (toute période)."
        on={cookies} set={setCookies} />
      <Choice label="Fichiers en cache" detail="Images et pages gardées pour aller plus vite." on={cache}
        set={setCache} />
      <div className="flex justify-end px-2 pt-1">
        <PushButton tone="danger" disabled={nothing}
          onClick={() => send({ kind: 'clearBrowsingData', since: since(period), history, cookies, cache })}>
          Effacer
        </PushButton>
      </div>
    </section>
  )
}
