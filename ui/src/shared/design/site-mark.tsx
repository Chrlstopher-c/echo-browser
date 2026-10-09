// Responsabilite : marque d'un site — favicon s'il se charge, sinon l'initiale de l'hote sur une pastille teintee
// (teinte stable par hote), la marque d'Echo pour ses pages, une feuille pour un fichier local.

import { useState, type ReactElement } from 'react'
import { readUrl } from '../url-shape'
import { IconFile, IconGlobe } from './icons'

export interface SiteMarkProps {
  url: string
  favicon: string | null
  /** Cote en pixels. */
  size: number
  className?: string
}

/** Teinte stable d'un hote : la meme pastille d'une session a l'autre. */
function hueOf(host: string): number {
  let hash = 0
  for (const char of host) hash = (hash * 31 + char.charCodeAt(0)) % 360
  return hash
}

function Initial({ host, size, className }: { host: string; size: number; className: string }): ReactElement {
  const hue = hueOf(host)
  return (
    <span
      data-mark="initiale"
      style={{
        width: size, height: size, fontSize: Math.round(size * 0.58),
        background: `hsl(${hue} 55% 55% / 0.28)`, color: `hsl(${hue} 55% 52%)`,
      }}
      className={`grid shrink-0 place-items-center rounded-[4px] leading-none font-semibold uppercase
        ${className}`}
    >
      {host.replace(/^www\./, '').charAt(0)}
    </span>
  )
}

export function SiteMark({ url, favicon, size, className = '' }: SiteMarkProps): ReactElement {
  const [broken, setBroken] = useState<string | null>(null)
  // L'initiale reste visible tant que l'icone n'est pas reellement chargee : une icone qui ne repond jamais ne laisse
  // pas de tuile vide (audit du 08/10).
  const [loaded, setLoaded] = useState<string | null>(null)
  if (url.startsWith('echo://')) {
    return (
      <span data-mark="echo" style={{ width: size, height: size, fontSize: Math.round(size * 0.6) }}
        className={`grid shrink-0 place-items-center rounded-[4px] bg-guard/20 font-bold leading-none text-guard
          ${className}`}>
        E
      </span>
    )
  }
  if (url.startsWith('file:')) {
    return <IconFile size={size} className={`shrink-0 text-ink-muted ${className}`} />
  }
  const shape = readUrl(url)
  const fallback = shape.host.length === 0
    ? <IconGlobe size={size} className={`shrink-0 text-ink-faint ${className}`} />
    : <Initial host={shape.host} size={size} className={className} />
  if (favicon === null || broken === favicon) return fallback
  const ready = loaded === favicon
  return (
    <span style={{ width: size, height: size }} className={`relative grid shrink-0 place-items-center ${className}`}>
      {!ready && fallback}
      <img src={favicon} alt="" width={size} height={size} draggable={false}
        onLoad={() => setLoaded(favicon)} onError={() => setBroken(favicon)}
        className={`absolute inset-0 rounded-[3px] ${ready ? '' : 'opacity-0'}`} />
    </span>
  )
}
