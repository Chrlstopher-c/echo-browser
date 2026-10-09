// Responsabilite : formulaire du compte Echo — creer un compte ou se connecter. Le mot de passe part vers le coeur
// d'Echo (sur la machine), qui en derive les cles : il n'est jamais envoye au service.

import { useState, type FormEvent, type ReactElement } from 'react'
import type { AccountView, UiRequest } from '../shared/contract'
import { PushButton } from '../shared/design/push-button'

export interface AccountFormProps {
  account: AccountView
  send: (request: UiRequest) => void
  /** Commencer par la creation de compte (premier lancement) plutot que la connexion. */
  createFirst?: boolean
}

const FIELD = `h-9 w-full rounded-row bg-shell px-3 text-[12.5px] text-ink shadow-field outline-none
  placeholder:text-ink-faint focus:ring-1 focus:ring-tint/50`

function problem(create: boolean, email: string, password: string, confirm: string): string | null {
  if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(email.trim())) return 'Adresse e-mail invalide.'
  if (create && password.length < 10) return 'Mot de passe : 10 caractères au moins.'
  if (create && password !== confirm) return 'Les deux mots de passe diffèrent.'
  return null
}

/** Solidite indicative, d'apres la longueur (le seul critere qui compte vraiment pour une phrase de passe). */
function Strength({ length }: { length: number }): ReactElement {
  const [label, tone] = length >= 14 ? ['Mot de passe solide', 'text-guard']
    : length >= 10 ? ['Mot de passe correct', 'text-warn'] : ['Trop court (10 caractères au moins)', 'text-danger']
  return <p className={`text-[11px] ${tone}`}>{label}</p>
}

export function AccountForm({ account, send, createFirst = false }: AccountFormProps): ReactElement {
  const [create, setCreate] = useState(createFirst)
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [local, setLocal] = useState<string | null>(null)
  const [shown, setShown] = useState(false)
  const kind = shown ? 'text' : 'password'
  const submit = (event: FormEvent): void => {
    event.preventDefault()
    const issue = problem(create, email, password, confirm)
    setLocal(issue)
    if (issue === null) send({ kind: 'accountSignIn', email: email.trim(), password, create })
  }
  const error = local ?? account.error
  return (
    <form onSubmit={submit} className="flex flex-col gap-2" aria-label={create ? 'Créer un compte' : 'Se connecter'}>
      <input className={FIELD} type="email" placeholder="Adresse e-mail" autoComplete="email" value={email}
        onChange={(e) => setEmail(e.target.value)} />
      <input className={FIELD} type={kind} placeholder="Mot de passe" value={password}
        autoComplete={create ? 'new-password' : 'current-password'} onChange={(e) => setPassword(e.target.value)} />
      {create && (
        <input className={FIELD} type={kind} placeholder="Confirmer le mot de passe" autoComplete="new-password"
          value={confirm} onChange={(e) => setConfirm(e.target.value)} />
      )}
      <label className="flex items-center gap-1.5 self-start text-[11.5px] text-ink-muted">
        <input type="checkbox" checked={shown} onChange={(e) => setShown(e.target.checked)} />
        Afficher le mot de passe
      </label>
      {create && password.length > 0 && <Strength length={password.length} />}
      {error !== null && <p className="text-[11.5px] text-danger">{error}</p>}
      <div className="flex items-center gap-2">
        <button type="submit" disabled={account.busy}
          className="h-8 rounded-row bg-card px-4 text-[12px] font-medium text-ink shadow-card transition-shadow
            hover:shadow-lift disabled:opacity-60">
          {account.busy ? 'Un instant…' : create ? 'Créer mon compte' : 'Se connecter'}
        </button>
        <PushButton onClick={() => { setCreate(!create); setLocal(null) }}>
          {create ? 'J’ai déjà un compte' : 'Créer un compte'}
        </PushButton>
      </div>
      <p className="text-[11px] leading-snug text-ink-faint">
        Vos données sont chiffrées sur cette machine avant d’être envoyées : sans votre mot de passe, personne ne peut
        les lire, pas même Echo. Un mot de passe oublié ne se récupère pas.
      </p>
    </form>
  )
}
