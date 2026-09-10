// Responsabilite : indicateur de securite du champ d'adresse, d'apres l'etat que le coeur connait.

import type { ReactElement } from 'react'
import type { Security } from '../shared/contract'
import { IconGlobe, IconLock, IconLockOpen, IconWarning, type IconComponent } from '../shared/design/icons'

export type SecurityReading = Security | 'blank'

interface Reading {
  label: string
  tone: string
  Icon: IconComponent
}

const READING: Record<SecurityReading, Reading> = {
  secure: { label: 'Connexion chiffrée', tone: 'text-ink-muted', Icon: IconLock },
  mixed: { label: 'Connexion chiffrée, contenu mixte', tone: 'text-warn', Icon: IconWarning },
  invalid: { label: 'Certificat invalide', tone: 'text-danger', Icon: IconWarning },
  insecure: { label: 'Connexion non chiffrée', tone: 'text-warn', Icon: IconLockOpen },
  local: { label: 'Page interne du navigateur', tone: 'text-ink-faint', Icon: IconGlobe },
  blank: { label: 'Aucune page chargée', tone: 'text-ink-faint', Icon: IconGlobe },
}

export function SecurityMark({ security, size = 13 }: { security: SecurityReading; size?: number }): ReactElement {
  const { label, tone, Icon } = READING[security]
  return (
    <span title={label} aria-label={label} className={`grid shrink-0 place-items-center ${tone}`}>
      <Icon size={size} />
    </span>
  )
}
