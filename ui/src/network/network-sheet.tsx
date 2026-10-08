// Responsabilite : feuille Reseau — ce que charge l'onglet actif, domaine par domaine (requetes, octets, tiers,
// blocages), le detail d'un domaine, et les actions a la volee : bloquer un domaine sur ce site, isolement strict.

import { useEffect, useState, type ReactElement } from 'react'
import type { NetDomainView, NetRequestView, NetworkView, UiRequest } from '../shared/contract'
import { IconChevronLeft } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'
import { Segmented } from '../shared/design/segmented'
import { Toggle } from '../shared/design/toggle'
import { blockedLabel, kindLabel, size } from './network-format'
import { NetworkJournal } from './network-journal'

export interface NetworkSheetProps {
  network: NetworkView | null
  send: (request: UiRequest) => void
}

interface DomainRowProps {
  d: NetDomainView
  blocked: boolean
  send: NetworkSheetProps['send']
}

function DomainRow({ d, blocked, send }: DomainRowProps): ReactElement {
  return (
    <div className="group flex items-center gap-2 rounded-row px-2 py-1.5 hover:bg-hover">
      <button type="button" className="min-w-0 flex-1 text-left"
        onClick={() => send({ kind: 'networkFocus', host: d.host })}>
        <p className={`truncate text-[12px] ${blocked ? 'text-danger line-through' : 'text-ink'}`}>{d.host}</p>
        <p className="numerique truncate text-[10.5px] text-ink-faint">
          {d.requests} req. · {size(d.bytes)}{d.blocked > 0 ? ` · ${d.blocked} bloquée(s)` : ''}
          {d.thirdParty ? ' · tiers' : ''}
        </p>
      </button>
      {d.thirdParty && (
        <PushButton tone={blocked ? 'neutral' : 'danger'}
          onClick={() => send({ kind: 'networkBlockHost', host: d.host, blocked: !blocked })}>
          {blocked ? 'Débloquer' : 'Bloquer'}
        </PushButton>
      )}
    </div>
  )
}

function RequestRow({ r }: { r: NetRequestView }): ReactElement {
  const path = r.url.replace(/^[a-z]+:\/\/[^/]+/, '') || '/'
  return (
    <div className="px-2 py-1">
      <p className={`truncate text-[11.5px] ${r.blocked !== null ? 'text-danger' : 'text-ink'}`} title={r.url}>
        {path}
      </p>
      <p className="numerique truncate text-[10.5px] text-ink-faint">
        {kindLabel(r.kind)} · {r.blocked !== null ? blockedLabel(r.blocked) : `${r.status ?? '…'} · ${size(r.bytes)}`}
        {r.durationMs !== null && r.blocked === null ? ` · ${r.durationMs} ms` : ''}
      </p>
    </div>
  )
}

function Detail({ network, send }: { network: NetworkView; send: NetworkSheetProps['send'] }): ReactElement {
  return (
    <div className="flex flex-col">
      <button type="button" onClick={() => send({ kind: 'networkFocus', host: null })}
        className="mb-1 flex items-center gap-1 px-2 text-[11.5px] text-ink-muted hover:text-ink">
        <IconChevronLeft size={12} /> Tous les domaines
      </button>
      <p className="truncate px-2 pb-1 text-[12.5px] font-medium text-ink">{network.focus}</p>
      {network.requests.map((r, i) => <RequestRow key={`${r.url}-${i}`} r={r} />)}
    </div>
  )
}

type View = 'domaines' | 'journal'

function Totals({ network }: { network: NetworkView }): ReactElement {
  const requests = network.domains.reduce((n, d) => n + d.requests, 0)
  const blocked = network.domains.reduce((n, d) => n + d.blocked, 0)
  const cells = [['Requêtes', `${requests}`], ['Reçu', size(network.totalBytes)], ['Bloquées', `${blocked}`]]
  return (
    <div className="grid grid-cols-3 gap-2 px-2">
      {cells.map(([label, value]) => (
        <div key={label} className="rounded-row bg-card px-2 py-1.5 shadow-card">
          <p className="text-[10px] text-ink-faint">{label}</p>
          <p className="numerique text-[13px] text-ink">{value}</p>
        </div>
      ))}
    </div>
  )
}

function Strict({ network, send }: { network: NetworkView; send: NetworkSheetProps['send'] }): ReactElement {
  const third = network.domains.filter((d) => d.thirdParty).length
  return (
    <div className="flex items-center justify-between gap-3 px-2">
      <div className="min-w-0">
        <p className="text-[12.5px] text-ink">Isolement strict</p>
        <p className="text-[11px] leading-snug text-ink-faint">
          Aucune requête vers un autre site ({third} tiers ici).
        </p>
      </div>
      <Toggle checked={network.strict} label="Isolement strict"
        onChange={(strict) => send({ kind: 'networkSetStrict', strict })} />
    </div>
  )
}

function Domains({ network, send }: { network: NetworkView; send: NetworkSheetProps['send'] }): ReactElement {
  if (network.focus !== null) return <Detail network={network} send={send} />
  return (
    <div className="flex flex-col">
      {network.domains.length === 0 && <p className="px-2 text-[11.5px] text-ink-faint">Aucune requête.</p>}
      {network.domains.map((d) => (
        <DomainRow key={d.host} d={d} blocked={network.blockedHosts.includes(d.host)} send={send} />
      ))}
    </div>
  )
}

export function NetworkSheet({ network, send }: NetworkSheetProps): ReactElement {
  const [view, setView] = useState<View>('domaines')
  useEffect(() => {
    send({ kind: 'networkWatch', on: true })
    return () => send({ kind: 'networkWatch', on: false })
  }, [send])
  if (network === null) return <p className="px-2 py-2 text-[11.5px] text-ink-faint">Lecture du réseau…</p>
  return (
    <div className="flex flex-col gap-2">
      <Totals network={network} />
      <Strict network={network} send={send} />
      <div className="px-2">
        <Segmented<View> name="reseau-vue" value={view} onChange={setView}
          segments={[{ id: 'domaines', label: 'Domaines' }, { id: 'journal', label: 'Journal' }]} />
      </div>
      {view === 'journal' ? <NetworkJournal journal={network.journal} /> : <Domains network={network} send={send} />}
    </div>
  )
}
