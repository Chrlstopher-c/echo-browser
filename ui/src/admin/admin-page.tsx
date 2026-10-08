// Responsabilite : page Administration d'Echo (comptes administrateurs) — chiffres du service, activite, coffre par
// type, versions, routes, usage (actifs, synchros, envois, plus actifs), et les comptes avec leur fiche. Jamais le
// contenu des coffres : il est chiffre sur les machines.

import type { ReactElement } from 'react'
import { IconRefresh } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'
import { AdminAccounts } from './admin-accounts'
import { DayBars, Gauges, Panel, Tile } from './admin-charts'
import type { AdminSummary } from './admin-data'
import { bytes, count, kindLabel } from './admin-format'
import type { AdminController } from './use-admin'

function Tiles({ s }: { s: AdminSummary }): ReactElement {
  const stored = s.storage.reduce((total, k) => total + k.bytes, 0)
  const signups = s.signups.reduce((total, d) => total + d.count, 0)
  return (
    <div className="grid grid-cols-2 gap-3 md:grid-cols-5">
      <Tile label="Comptes" value={count(s.accounts)} detail={`${signups} inscrits sur 30 j`} />
      <Tile label="Actifs 7 j" value={count(s.active.week)}
        detail={`${s.active.day} sur 24 h · ${s.active.month} sur 30 j`} />
      <Tile label="Machines" value={count(s.machines.total)}
        detail={`${s.machines.active} vues sur 7 j · ${s.connected} comptes connectés`} />
      <Tile label="Coffre chiffré" value={bytes(stored)} detail={`${s.storage.length} types`} />
      <Tile label="Échecs de connexion" value={count(s.loginFailures)} detail="sur 24 h" />
    </div>
  )
}

function Usage({ s, open }: { s: AdminSummary; open: (id: string) => void }): ReactElement {
  return (
    <div className="grid gap-3 md:grid-cols-2">
      <Panel title="Utilisateurs actifs par jour"><DayBars days={s.activeByDay} /></Panel>
      <Panel title="Synchronisations par jour"><DayBars days={s.syncsByDay} /></Panel>
      <Panel title="Envois par type · 30 jours">
        <Gauges rows={s.writesByType.map((w) => ({ ...w, name: kindLabel(w.name) }))} format={count} />
      </Panel>
      <Panel title="Plus actifs · 7 jours">
        {s.topUsers.length === 0 ? <p className="text-[11.5px] text-ink-faint">Rien pour l’instant.</p> : (
          <div className="flex flex-col gap-1">
            {s.topUsers.map((u) => (
              <button key={u.id} type="button" onClick={() => open(u.id)}
                className="flex items-center gap-3 rounded-row px-1 py-0.5 text-left text-[12px] hover:bg-hover">
                <span className="flex-1 truncate text-ink">{u.name}</span>
                <span className="numerique text-ink-muted">{count(u.count)} requêtes</span>
              </button>
            ))}
          </div>
        )}
      </Panel>
    </div>
  )
}

function Summary({ s }: { s: AdminSummary }): ReactElement {
  const storage = s.storage.map((k) => ({ name: kindLabel(k.name), count: k.bytes }))
  return (
    <>
      <Tiles s={s} />
      <div className="grid gap-3 md:grid-cols-2">
        <Panel title="Inscriptions · 30 jours"><DayBars days={s.signups} /></Panel>
        <Panel title={<>Requêtes · 30 jours <span className="text-danger">· erreurs</span></>}>
          <DayBars days={s.requests} />
        </Panel>
        <Panel title="Versions d’Echo · actifs 30 jours">
          <Gauges rows={s.versions.map((v) => ({ ...v, name: `Echo ${v.name}` }))} format={count} />
        </Panel>
        <Panel title="Coffre par type"><Gauges rows={storage} format={bytes} /></Panel>
      </div>
      <Panel title="Routes · 7 jours"><Gauges rows={s.routes} format={count} /></Panel>
    </>
  )
}

export function AdminPage({ admin }: { admin: AdminController }): ReactElement {
  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <p className="flex-1 text-[11.5px] leading-snug text-ink-faint">
          Les données des utilisateurs sont chiffrées sur leurs machines : cette page ne peut pas les lire.
        </p>
        <PushButton icon={<IconRefresh size={12} />} onClick={admin.refresh}>Rafraîchir</PushButton>
      </div>
      {admin.error !== null && <p className="text-[12px] text-danger">{admin.error}</p>}
      {admin.summary === null
        ? admin.error === null && <p className="text-[12px] text-ink-faint">Lecture du service…</p>
        : <><Summary s={admin.summary} /><Usage s={admin.summary} open={admin.open} /></>}
      <Panel title="Comptes"><AdminAccounts admin={admin} /></Panel>
    </div>
  )
}
