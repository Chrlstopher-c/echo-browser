// Responsabilite : marque d'un site — favicon s'il existe, sinon l'initiale de l'hote, sinon un globe.

import type { ReactElement } from 'react'
import { readUrl } from '../url-shape'
import { IconGlobe } from './icons'

export interface SiteMarkProps {
  url: string
  favicon: string | null
  /** Cote en pixels. */
  size: number
  className?: string
}

export function SiteMark({ url, favicon, size, className = '' }: SiteMarkProps): ReactElement {
  if (favicon !== null) {
    return (
      <img src={favicon} alt="" width={size} height={size} draggable={false}
        className={`shrink-0 rounded-[3px] ${className}`} />
    )
  }
  const shape = readUrl(url)
  if (shape.host.length === 0 || shape.safety === 'local') {
    return <IconGlobe size={size} className={`shrink-0 text-ink-faint ${className}`} />
  }
  return (
    <span
      style={{ width: size, height: size, fontSize: Math.round(size * 0.56) }}
      className={`numerique grid shrink-0 place-items-center rounded-[4px] bg-ink/10 leading-none
        font-medium text-ink-muted uppercase ${className}`}
    >
      {shape.host.charAt(0)}
    </span>
  )
}
