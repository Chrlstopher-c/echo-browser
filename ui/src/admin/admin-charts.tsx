// Responsabilite : les visuels du tableau de bord — tuiles chiffrees, barres par jour sur 30 jours, jauges par nom.

import type { ReactElement } from 'react'
import type { DayCount, NamedCount } from './admin-data'

const DAYS = 30
const DAY_MS = 86_400_000

export function Tile({ label, value, detail }: { label: string; value: string; detail: string }): ReactElement {
  return (
    <div className="rounded-tile bg-card p-3.5 shadow-card">
      <p className="text-[11.5px] text-ink-muted">{label}</p>
      <p className="numerique text-[22px] font-semibold tracking-tight text-ink">{value}</p>
      <p className="text-[10.5px] text-ink-faint">{detail}</p>
    </div>
  )
}

export function Panel({ title, children }: { title: ReactElement | string; children: ReactElement }): ReactElement {
  return (
    <section className="rounded-tile bg-card p-4 shadow-card">
      <h2 className="mb-3 text-[10.5px] font-semibold tracking-[0.12em] text-ink-faint uppercase">{title}</h2>
      {children}
    </section>
  )
}

/** Une barre par jour (les jours sans valeur a zero) ; la part d'erreurs en rouge au pied. */
export function DayBars({ days }: { days: DayCount[] }): ReactElement {
  const byDay = new Map(days.map((d) => [d.day, d]))
  const span = Array.from({ length: DAYS },
    (_, i) => new Date(Date.now() - (DAYS - 1 - i) * DAY_MS).toISOString().slice(0, 10))
  const max = Math.max(1, ...days.map((d) => d.count))
  return (
    <div>
      <div className="flex h-24 items-end gap-[3px]">
        {span.map((day) => {
          const d = byDay.get(day)
          const count = d?.count ?? 0
          const errors = d?.errors ?? 0
          return (
            <div key={day} title={`${day} : ${count}${errors > 0 ? ` (${errors} erreurs)` : ''}`}
              className="relative min-h-[2px] flex-1 rounded-t-[3px] bg-tint"
              style={{ height: `${(count / max) * 100}%` }}>
              {errors > 0 && (
                <div className="absolute inset-x-0 bottom-0 rounded-t-[3px] bg-danger"
                  style={{ height: `${(errors / count) * 100}%` }} />
              )}
            </div>
          )
        })}
      </div>
      <div className="mt-1.5 flex justify-between text-[10px] text-ink-faint">
        <span>{span[0]?.slice(5)}</span>
        <span>aujourd’hui</span>
      </div>
    </div>
  )
}

export function Gauges({ rows, format }: { rows: NamedCount[]; format: (value: number) => string }): ReactElement {
  if (rows.length === 0) return <p className="text-[11.5px] text-ink-faint">Rien pour l’instant.</p>
  const max = Math.max(1, ...rows.map((r) => r.count))
  return (
    <div className="flex flex-col gap-1.5">
      {rows.map((row) => (
        <div key={row.name} className="flex items-center gap-3 text-[12px]">
          <span className="w-2/5 truncate text-ink">{row.name}</span>
          <div className="h-1.5 flex-1 rounded-full bg-field shadow-field">
            <div className="h-full rounded-full bg-tint" style={{ width: `${(row.count / max) * 100}%` }} />
          </div>
          <span className="numerique w-16 text-right text-ink-muted">{format(row.count)}</span>
        </div>
      ))}
    </div>
  )
}
