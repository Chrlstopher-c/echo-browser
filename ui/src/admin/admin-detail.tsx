// Responsabilite : fiche d'un compte dans l'administration — requetes par jour (30 j), actions, machines connectees,
// coffre par type. Des comptages et des dates : jamais le contenu.

import type { ReactElement } from 'react'
import type { AdminDetail } from './admin-data'
import { DayBars, Gauges } from './admin-charts'
import { actionLabel, bytes, count, kindLabel, when } from './admin-format'

function Block({ title, children }: { title: string; children: ReactElement }): ReactElement {
  return (
    <div className="rounded-row bg-field p-3 shadow-field">
      <p className="mb-2 text-[10px] font-semibold tracking-[0.12em] text-ink-faint uppercase">{title}</p>
      {children}
    </div>
  )
}

function Machines({ detail }: { detail: AdminDetail }): ReactElement {
  if (detail.machines.length === 0) return <p className="text-[11.5px] text-ink-faint">Aucune machine connectée.</p>
  return (
    <div className="flex flex-col gap-1">
      {detail.machines.map((m, index) => (
        <p key={`${m.createdAt}-${index}`} className="numerique text-[11px] text-ink-muted">
          <span className="text-ink">Machine {index + 1}</span> · {m.version ?? 'version inconnue'} · connectée
          {' '}{when(m.createdAt)} · vue {when(m.seenAt)}{m.expiresAt === null ? ' · session fermée' : ''}
        </p>
      ))}
    </div>
  )
}

export function AccountDetail({ detail }: { detail: AdminDetail | null }): ReactElement {
  if (detail === null) return <p className="px-1 py-2 text-[11.5px] text-ink-faint">Lecture de la fiche…</p>
  const total = detail.byDay.reduce((sum, d) => sum + d.count, 0)
  return (
    <div className="grid gap-2 pb-3 md:grid-cols-2">
      <Block title={`Requêtes · 30 jours · ${count(total)}`}><DayBars days={detail.byDay} /></Block>
      <Block title="Actions · 30 jours">
        <Gauges rows={detail.byAction.map((a) => ({ ...a, name: actionLabel(a.name) }))} format={count} />
      </Block>
      <Block title={`Machines · ${detail.machines.length}`}><Machines detail={detail} /></Block>
      <Block title="Coffre">
        <Gauges rows={detail.vault.map((v) => ({ name: `${kindLabel(v.type)} · v${v.version} · ${when(v.updatedAt)}`,
          count: v.bytes }))} format={bytes} />
      </Block>
    </div>
  )
}
