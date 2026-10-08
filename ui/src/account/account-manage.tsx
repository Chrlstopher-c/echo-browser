// Responsabilite : Reglages → Compte, une fois connecte — etat de la synchronisation, historique synchronise ou non,
// donnees stockees sur le serveur, et suppression complete du compte.

import { useState, type ReactElement } from 'react'
import type { AccountView, UiRequest, VaultKindView } from '../shared/contract'
import { ConfirmStrip } from '../shared/design/confirm-strip'
import { IconRefresh, IconTrash } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'
import { Toggle } from '../shared/design/toggle'
import { VaultPanel } from './vault-panel'

function since(seconds: number | null): string {
  if (seconds === null) return 'pas encore synchronisé'
  const minutes = Math.round((Date.now() / 1000 - seconds) / 60)
  if (minutes < 1) return 'synchronisé à l’instant'
  if (minutes < 60) return `synchronisé il y a ${minutes} min`
  return `synchronisé il y a ${Math.round(minutes / 60)} h`
}

export interface AccountManageProps {
  account: AccountView & { email: string }
  vault: VaultKindView[] | null
  send: (request: UiRequest) => void
}

function Row({ title, detail, children }: { title: string; detail: string; children: ReactElement }): ReactElement {
  return (
    <div className="flex items-center justify-between gap-3 px-2 py-2">
      <div className="min-w-0">
        <p className="text-[12.5px] text-ink">{title}</p>
        <p className="text-[11px] leading-snug text-ink-faint">{detail}</p>
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  )
}

function DeleteRow({ send }: { send: AccountManageProps['send'] }): ReactElement {
  const [asking, setAsking] = useState(false)
  if (asking) {
    return (
      <ConfirmStrip
        question="Le compte et tout ce que le serveur en garde seront effacés, sans retour. Ce qui est sur cette machine reste."
        confirmLabel="Tout supprimer"
        onConfirm={() => { setAsking(false); send({ kind: 'accountDelete' }) }}
        onCancel={() => setAsking(false)}
      />
    )
  }
  return (
    <Row title="Supprimer le compte" detail="Efface le compte et toutes ses données du serveur.">
      <PushButton tone="danger" icon={<IconTrash size={12} />} onClick={() => setAsking(true)}>Supprimer</PushButton>
    </Row>
  )
}

export function AccountManage({ account, vault, send }: AccountManageProps): ReactElement {
  const [showVault, setShowVault] = useState(false)
  const toggleVault = (): void => {
    if (!showVault) send({ kind: 'accountInspect' })
    setShowVault(!showVault)
  }
  return (
    <div className="flex flex-col divide-y divide-hairline">
      <div className="flex items-center gap-2 px-2 py-1">
        <div className="min-w-0 flex-1">
          <p className="truncate text-[12.5px] text-ink">{account.email}</p>
          <p className="text-[11px] text-ink-faint">{account.busy ? 'Synchronisation…' : since(account.lastSync)}</p>
          {account.error !== null && <p className="text-[11px] text-danger">{account.error}</p>}
        </div>
        <PushButton disabled={account.busy} onClick={() => send({ kind: 'accountSync' })}
          icon={<IconRefresh size={12} />}>
          Synchroniser
        </PushButton>
        <PushButton onClick={() => send({ kind: 'accountSignOut' })}>Se déconnecter</PushButton>
      </div>
      <Row title="Synchroniser l’historique" detail="Les 1 000 dernières adresses visitées, chiffrées de bout en bout.">
        <Toggle checked={account.history} label="Synchroniser l’historique"
          onChange={(next) => send({ kind: 'updateSetting', key: 'sync.history', value: { type: 'flag', value: next } })} />
      </Row>
      <div>
        <Row title="Données stockées" detail="Ce que le serveur garde de ce compte, déchiffré ici.">
          <PushButton disabled={account.busy && !showVault} onClick={toggleVault}>
            {showVault ? 'Masquer' : 'Afficher'}
          </PushButton>
        </Row>
        {showVault && (vault === null || account.busy
          ? <p className="px-2 pb-2 text-[11px] text-ink-faint">Lecture du serveur…</p>
          : <VaultPanel kinds={vault} />)}
      </div>
      {account.admin && (
        <Row title="Administration" detail="Tableau de bord du service : comptes, usage, santé.">
          <PushButton onClick={() => send({ kind: 'openPage', page: 'admin' })}>Ouvrir</PushButton>
        </Row>
      )}
      <DeleteRow send={send} />
    </div>
  )
}
