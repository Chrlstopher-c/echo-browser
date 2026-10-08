// Responsabilite : Administration → signaux partages — ce que les navigateurs ayant choisi de partager envoient
// (lots par jour, versions), et les domaines et traqueurs vus par assez d'installations pour rester anonymes.

import type { ReactElement } from 'react'
import { DayBars, Gauges, Panel } from './admin-charts'
import type { AdminSignals, SignalKey } from './admin-data'
import { count } from './admin-format'

function Keys({ rows, empty }: { rows: SignalKey[]; empty: string }): ReactElement {
  if (rows.length === 0) return <p className="text-[11.5px] text-ink-faint">{empty}</p>
  return <Gauges rows={rows.map((r) => ({ name: `${r.key} · ${r.installs} inst. · ${r.days} j`, count: r.total }))}
    format={count} />
}

export function AdminSignalsPanel({ signals }: { signals: AdminSignals | null }): ReactElement {
  if (signals === null) return <p className="text-[11.5px] text-ink-faint">Signaux indisponibles.</p>
  const below = `Rien au-delà du seuil (${signals.threshold} installations) pour l’instant.`
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[11.5px] leading-snug text-ink-faint">
        Origine : navigateurs Echo dont l’utilisateur a activé « Partager des signaux anonymes » (Réglages → Vie
        privée). Un lot par installation et par jour, sans compte ni identifiant stable, domaines seulement ; une clé
        n’apparaît qu’à partir de {signals.threshold} installations un même jour.
      </p>
      <div className="grid gap-3 md:grid-cols-2">
        <Panel title="Installations qui partagent · par jour"><DayBars days={signals.batchesByDay} /></Panel>
        <Panel title="Versions qui partagent">
          <Gauges rows={signals.versions.map((v) => ({ ...v, name: `Echo ${v.name}` }))} format={count} />
        </Panel>
        <Panel title="Sites les plus visités · 30 jours"><Keys rows={signals.sites} empty={below} /></Panel>
        <Panel title="Traqueurs les plus bloqués · 30 jours"><Keys rows={signals.blocked} empty={below} /></Panel>
      </div>
    </div>
  )
}
