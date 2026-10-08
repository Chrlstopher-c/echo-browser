// Responsabilite : la liste des comptes de l'administration — e-mail, dates, version, taille du coffre, sessions — et
// les actions : rendre ou retirer admin, deconnecter partout, supprimer (confirmation en place).

import { useState, type ReactElement } from 'react'
import { ConfirmStrip } from '../shared/design/confirm-strip'
import { PushButton } from '../shared/design/push-button'
import { SearchField } from '../shared/design/search-field'
import type { AdminAccount } from './admin-data'
import { bytes, when } from './admin-format'
import { AccountDetail } from './admin-detail'
import type { AdminController } from './use-admin'

function AccountRow({ account, admin }: { account: AdminAccount; admin: AdminController }): ReactElement {
  const [asking, setAsking] = useState(false)
  if (asking) {
    return (
      <ConfirmStrip question={`Supprimer ${account.email} et tout son coffre ? Sans retour.`}
        confirmLabel="Supprimer définitivement" onCancel={() => setAsking(false)}
        onConfirm={() => { setAsking(false); admin.remove(account.id) }} />
    )
  }
  const opened = admin.opened === account.id
  return (
    <div className="border-t border-hairline">
      <div className="flex items-center gap-3 px-1 py-2">
        <button type="button" className="min-w-0 flex-1 text-left" title="Voir la fiche"
          onClick={() => admin.open(opened ? null : account.id)}>
          <p className="truncate text-[12.5px] text-ink">
            {account.email}
            {account.admin && <span className="ml-2 rounded-md px-1.5 text-[10px] text-guard shadow-field">admin</span>}
          </p>
          <p className="numerique truncate text-[10.5px] text-ink-faint">
            créé {when(account.createdAt)} · vu {when(account.seenAt)} · {account.version ?? 'version inconnue'} ·
            {' '}{bytes(account.bytes)} · {account.sessions} session(s)
          </p>
        </button>
        <PushButton onClick={() => admin.setAdmin(account.id, !account.admin)}>
          {account.admin ? 'Retirer admin' : 'Rendre admin'}
        </PushButton>
        <PushButton onClick={() => admin.signOut(account.id)}>Déconnecter</PushButton>
        <PushButton tone="danger" onClick={() => setAsking(true)}>Supprimer</PushButton>
      </div>
      {opened && <AccountDetail detail={admin.detail} />}
    </div>
  )
}

export function AdminAccounts({ admin }: { admin: AdminController }): ReactElement {
  return (
    <div className="flex flex-col gap-2">
      <SearchField value={admin.query} placeholder="Chercher une adresse e-mail" onChange={admin.search} />
      {admin.accounts.length === 0
        ? <p className="px-1 py-2 text-[11.5px] text-ink-faint">Aucun compte.</p>
        : <div>{admin.accounts.map((a) => <AccountRow key={a.id} account={a} admin={admin} />)}</div>}
    </div>
  )
}
