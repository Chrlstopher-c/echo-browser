// Responsabilite : indicateur de securite du champ d'adresse.

import type { ReactElement } from 'react'
import { IconGlobe, IconLock, IconWarning } from '../shared/design/icons'
import type { UrlSafety } from '../shared/url-shape'

const LABEL: Record<UrlSafety, string> = {
  secure: 'Connexion chiffrée',
  insecure: 'Connexion non chiffrée',
  local: 'Page interne du navigateur',
  blank: 'Aucune page chargée',
}

export function SecurityMark({ safety }: { safety: UrlSafety }): ReactElement {
  const shared = 'shrink-0'
  if (safety === 'secure') {
    return <span title={LABEL.secure}><IconLock size={13} className={`${shared} text-ink-muted`} /></span>
  }
  if (safety === 'insecure') {
    return <span title={LABEL.insecure}><IconWarning size={13} className={`${shared} text-warn`} /></span>
  }
  return <span title={LABEL[safety]}><IconGlobe size={13} className={`${shared} text-ink-faint`} /></span>
}
