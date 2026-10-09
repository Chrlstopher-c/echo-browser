// Responsabilite : indicateur de securite du champ d'adresse, d'apres l'etat que le coeur connait.

import type { ReactElement } from 'react'
import type { Security } from '../shared/contract'
import { IconLock, IconLockOpen, IconSearch, IconWarning, type IconComponent } from '../shared/design/icons'

export type SecurityReading = Security | 'blank'

interface Reading {
  label: string
  tone: string
  Icon: IconComponent
}

const READING: Record<SecurityReading, Reading> = {
  secure: { label: 'Connexion chiffrée', tone: 'text-ink-muted', Icon: IconLock },
  mixed: { label: 'Connexion chiffrée, contenu mixte', tone: 'text-warn', Icon: IconWarning },
  invalid: { label: 'Non sécurisé : certificat refusé', tone: 'text-danger', Icon: IconWarning },
  insecure: { label: 'Non sécurisé : connexion non chiffrée', tone: 'text-warn', Icon: IconLockOpen },
  local: { label: 'Page interne du navigateur', tone: 'text-ink-faint', Icon: IconSearch },
  failed: { label: 'Page non chargée', tone: 'text-ink-faint', Icon: IconWarning },
  blank: { label: 'Rechercher ou saisir une adresse', tone: 'text-ink-faint', Icon: IconSearch },
}

export interface SecurityMarkProps {
  security: SecurityReading
  size?: number
  /** Ouvre la securite du site (panneau Reseau) : avec qui la page communique, ce qui est bloque, le journal. */
  onOpen?: (() => void) | undefined
}

export function SecurityMark({ security, size = 13, onOpen }: SecurityMarkProps): ReactElement {
  const { label, tone, Icon } = READING[security]
  if (onOpen !== undefined && security !== 'blank' && security !== 'local') {
    return (
      <button type="button" title={`${label} — voir la sécurité du site`} aria-label="Sécurité du site" onClick={onOpen}
        className={`-m-1 grid shrink-0 place-items-center rounded-full p-1 transition-shadow hover:shadow-card ${tone}`}>
        <Icon size={size} />
      </button>
    )
  }
  return (
    <span title={label} aria-label={label} className={`grid shrink-0 place-items-center ${tone}`}>
      <Icon size={size} />
    </span>
  )
}
