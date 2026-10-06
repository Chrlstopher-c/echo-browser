// Responsabilite : lecture d'une URL pour l'affichage — hote, schema, sortie du champ d'adresse.

import type { Security } from './contract'

export type UrlSafety = 'secure' | 'insecure' | 'local' | 'blank'

export interface UrlShape {
  host: string
  path: string
  safety: UrlSafety
}

const LOCAL_SCHEMES = new Set(['about:', 'file:', 'echo:', 'chrome:', 'data:'])

/** Decompose une URL pour l'affichage. Ne jette jamais : une saisie libre reste affichable. */
export function readUrl(raw: string): UrlShape {
  if (raw.trim().length === 0) return { host: '', path: '', safety: 'blank' }
  try {
    const parsed = new URL(raw)
    // Pages internes (accueil, terminal) : rien a montrer, le champ reste une invite.
    if (parsed.protocol === 'echo:') return { host: '', path: '', safety: 'local' }
    if (LOCAL_SCHEMES.has(parsed.protocol)) {
      return { host: parsed.host || parsed.pathname, path: '', safety: 'local' }
    }
    const host = parsed.host.replace(/^www\./, '')
    const path = `${parsed.pathname === '/' ? '' : parsed.pathname}${parsed.search}`
    return { host, path, safety: parsed.protocol === 'https:' ? 'secure' : 'insecure' }
  } catch {
    return { host: raw, path: '', safety: 'blank' }
  }
}

/** Etiquette courte d'un onglet quand le titre manque encore. */
export function fallbackTitle(url: string): string {
  const shape = readUrl(url)
  return shape.host.length > 0 ? shape.host : 'Nouvel onglet'
}

/** Vrai si la saisie ressemble a une adresse plutot qu'a une recherche. */
export function looksLikeUrl(input: string): boolean {
  const value = input.trim()
  if (value.length === 0 || /\s/.test(value)) return false
  if (/^[a-z][a-z0-9+.-]*:\/\//i.test(value)) return true
  if (value.startsWith('about:') || value.startsWith('file:')) return true
  return /^[^./]+(\.[^./]+)+(\/|$|:\d)/.test(value)
}

/** Etat de securite deduit de l'URL — reserve au faux coeur, le vrai le connait. */
export function securityOf(url: string): Security {
  const safety = readUrl(url).safety
  if (safety === 'secure') return 'secure'
  if (safety === 'insecure') return 'insecure'
  return 'local'
}
