// Responsabilite : Reglages → Compte — l'etat de la synchronisation, et la connexion quand il n'y a pas de compte.

import type { ReactElement } from 'react'
import type { AccountView, UiRequest } from '../shared/contract'
import { IconRefresh } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'
import { SectionLabel } from '../shared/design/section-label'
import { AccountForm } from './account-form'

function since(seconds: number | null): string {
  if (seconds === null) return 'pas encore synchronisé'
  const minutes = Math.round((Date.now() / 1000 - seconds) / 60)
  if (minutes < 1) return 'synchronisé à l’instant'
  if (minutes < 60) return `synchronisé il y a ${minutes} min`
  return `synchronisé il y a ${Math.round(minutes / 60)} h`
}

export interface AccountSectionProps {
  account: AccountView
  send: (request: UiRequest) => void
}

export function AccountSection({ account, send }: AccountSectionProps): ReactElement | null {
  if (!account.available) return null
  return (
    <section>
      <SectionLabel>Compte Echo</SectionLabel>
      {account.email === null ? (
        <div className="px-2 py-1">
          <p className="mb-2 text-[12px] leading-snug text-ink-muted">
            Retrouvez réglages, favoris et extensions sur toutes vos machines.
          </p>
          <AccountForm account={account} send={send} />
        </div>
      ) : (
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
      )}
    </section>
  )
}
