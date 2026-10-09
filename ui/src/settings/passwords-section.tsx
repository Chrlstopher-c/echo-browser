// Responsabilite : mots de passe — Echo n'a pas de coffre a lui (un gestionnaire dedie est plus sur et suit
// partout) : il mene en un clic a Proton Pass ou Bitwarden, installes comme extensions dans le profil.

import type { ReactElement } from 'react'
import type { UiRequest } from '../shared/contract'
import { PushButton } from '../shared/design/push-button'
import { SectionLabel } from '../shared/design/section-label'

const MANAGERS = [
  { name: 'Proton Pass', url: 'https://chromewebstore.google.com/detail/ghmbeldphafepmbegfdlkpapadhbakde' },
  { name: 'Bitwarden', url: 'https://chromewebstore.google.com/detail/nngceckbapebfimnlniiiahkandclblb' },
]

export function PasswordsSection({ send }: { send: (request: UiRequest) => void }): ReactElement {
  return (
    <section>
      <SectionLabel>Mots de passe</SectionLabel>
      <p className="px-2 pb-2 text-[11.5px] leading-snug text-ink-muted">
        Echo confie les mots de passe à un gestionnaire dédié, chiffré et présent sur tous vos appareils. Un clic ouvre
        sa fiche : « Ajouter à Echo » l’installe dans ce profil (il s’active au prochain démarrage).
      </p>
      <div className="flex flex-wrap gap-2 px-2">
        {MANAGERS.map((manager) => (
          <PushButton key={manager.name} onClick={() => send({ kind: 'newTab', url: manager.url })}>
            Installer {manager.name}
          </PushButton>
        ))}
      </div>
    </section>
  )
}
