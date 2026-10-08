// Responsabilite : Reglages → Compte — la connexion quand il n'y a pas de compte, sa gestion sinon.

import type { ReactElement } from 'react'
import type { AccountView, UiRequest, VaultKindView } from '../shared/contract'
import { SectionLabel } from '../shared/design/section-label'
import { AccountForm } from './account-form'
import { AccountManage } from './account-manage'

export interface AccountSectionProps {
  account: AccountView
  vault: VaultKindView[] | null
  send: (request: UiRequest) => void
}

export function AccountSection({ account, vault, send }: AccountSectionProps): ReactElement | null {
  if (!account.available) return null
  return (
    <section>
      <SectionLabel>Compte Echo</SectionLabel>
      {account.email === null ? (
        <div className="px-2 py-1">
          <p className="mb-2 text-[12px] leading-snug text-ink-muted">
            Retrouvez réglages, favoris, extensions et historique sur toutes vos machines.
          </p>
          <AccountForm account={account} send={send} />
        </div>
      ) : (
        <AccountManage account={{ ...account, email: account.email }} vault={vault} send={send} />
      )}
    </section>
  )
}
