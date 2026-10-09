// Responsabilite : vue « Poids » du panneau Reseau — ce qui pese dans la page : part des tiers, poids par type, les
// requetes les plus lourdes et les plus lentes, et les alertes (image trop lourde, gros script tiers).

import type { ReactElement } from 'react'
import type { NetRequestView, NetWeightView } from '../shared/contract'
import { IconWarning } from '../shared/design/icons'
import { kindLabel, size } from './network-format'

const HEAVY_IMAGE = 500 * 1024
const HEAVY_THIRD_SCRIPT = 100 * 1024

function name(r: NetRequestView): string {
  const path = r.url.replace(/^[a-z]+:\/\/[^/]+/, '').split('?')[0] ?? ''
  return `${r.host}${path.length > 1 ? path : ''}`
}

/** Ce qui merite d'etre signale : images surdimensionnees, scripts tiers lourds. */
export function alerts(weight: NetWeightView): string[] {
  return weight.heaviest.flatMap((r) => {
    if (r.kind === 'image' && r.bytes > HEAVY_IMAGE) return [`Image lourde (${size(r.bytes)}) : ${name(r)}`]
    if (r.kind === 'script' && r.thirdParty && r.bytes > HEAVY_THIRD_SCRIPT) {
      return [`Script tiers lourd (${size(r.bytes)}) : ${name(r)}`]
    }
    return []
  })
}

interface RankedProps {
  title: string
  rows: NetRequestView[]
  value: (r: NetRequestView) => string
}

function Ranked({ title, rows, value }: RankedProps): ReactElement {
  return (
    <div className="flex flex-col">
      <p className="px-2 pb-1 text-[10.5px] font-semibold tracking-[0.12em] text-ink-faint uppercase">{title}</p>
      {rows.map((r, i) => (
        <div key={`${r.url}-${i}`} className="flex items-center gap-2 px-2 py-0.5" title={r.url}>
          <span className="min-w-0 flex-1 truncate text-[11.5px] text-ink">{name(r)}</span>
          <span className="numerique shrink-0 text-[10.5px] text-ink-muted">{value(r)}</span>
        </div>
      ))}
    </div>
  )
}

export function NetworkWeight({ weight, total }: { weight: NetWeightView; total: number }): ReactElement {
  const share = total > 0 ? Math.round((weight.thirdPartyBytes / total) * 100) : 0
  const warnings = alerts(weight)
  return (
    <div className="flex flex-col gap-3">
      <div className="px-2">
        <p className="text-[12px] text-ink">Tiers : <span className="numerique">{share} %</span> du poids reçu</p>
        <div className="mt-1 h-1.5 rounded-full bg-field shadow-field">
          <div className="h-full rounded-full bg-warn" style={{ width: `${share}%` }} />
        </div>
        <p className="mt-1.5 text-[10.5px] text-ink-faint">
          {weight.byKind.slice(0, 5).map(([kind, bytes]) => `${kindLabel(kind)} ${size(bytes)}`).join(' · ')}
        </p>
      </div>
      {warnings.length > 0 && (
        <div className="flex flex-col gap-1 px-2">
          {warnings.map((w) => (
            <p key={w} className="flex items-start gap-1.5 text-[11px] leading-snug text-ink">
              <IconWarning size={12} className="mt-px shrink-0 text-warn" />
              <span className="[overflow-wrap:anywhere]">{w}</span>
            </p>
          ))}
        </div>
      )}
      <Ranked title="Les plus lourdes" rows={weight.heaviest} value={(r) => size(r.bytes)} />
      <Ranked title="Les plus lentes" rows={weight.slowest} value={(r) => `${r.durationMs ?? 0} ms`} />
    </div>
  )
}
