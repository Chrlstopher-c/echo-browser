// Responsabilite : les espaces — une teinte (graphite, sable…) dans l'un des deux schemas, clair ou sombre.
// Le style est neumorphique : une seule matiere (`shell`), le relief vient d'un double ombrage — une
// lumiere (`hi`) en haut a gauche, une ombre (`lo`) en bas a droite. Tous les jetons se deduisent de la teinte.

export type SpaceId = 'graphite' | 'sable' | 'rose' | 'foret' | 'ardoise'

export type Scheme = 'dark' | 'light'

export interface SpaceTokens {
  /** Couleur plate de la barre, du cadre autour de la page, et de chaque element en relief. */
  shell: string
  /** Lueur en haut a gauche de la barre. */
  glow: string
  /** Surface d'un element en relief : la meme matiere que la barre. */
  card: string
  hover: string
  field: string
  hairline: string
  ink: string
  inkMuted: string
  inkFaint: string
  /** Teinte propre a l'espace : pastille du selecteur, accent des elements actifs. */
  tint: string
  /** Lumiere et ombre du relief. */
  hi: string
  lo: string
}

export interface Space {
  /** Le profil (identifiant libre) ; sa teinte est `hue`. */
  id: string
  hue: SpaceId
  name: string
  scheme: Scheme
  tokens: SpaceTokens
}

export const DEFAULT_SPACE: SpaceId = 'graphite'
export const DEFAULT_SCHEME: Scheme = 'dark'

interface Hue {
  id: SpaceId
  name: string
  /** Teinte en degres et saturation de fond (0 a 1) : tres faible, la matiere reste presque neutre. */
  h: number
  s: number
  /** Saturation de l'accent. */
  accent: number
}

export const HUES: readonly Hue[] = [
  { id: 'graphite', name: 'Graphite', h: 222, s: 0.06, accent: 0.14 },
  { id: 'sable', name: 'Sable', h: 36, s: 0.16, accent: 0.5 },
  { id: 'rose', name: 'Rose', h: 340, s: 0.14, accent: 0.45 },
  { id: 'foret', name: 'Forêt', h: 150, s: 0.14, accent: 0.4 },
  { id: 'ardoise', name: 'Ardoise', h: 215, s: 0.2, accent: 0.5 },
]

/** Teinte, saturation et luminosite (0 a 100) vers `#rrggbb`. */
function hsl(h: number, s: number, l: number): string {
  const light = l / 100
  const a = s * Math.min(light, 1 - light)
  const channel = (n: number): string => {
    const k = (n + h / 30) % 12
    const value = light - a * Math.max(-1, Math.min(k - 3, 9 - k, 1))
    return Math.round(value * 255).toString(16).padStart(2, '0')
  }
  return `#${channel(0)}${channel(8)}${channel(4)}`
}

function rgba(h: number, s: number, l: number, alpha: number): string {
  const hex = hsl(h, s, l)
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16))
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

function darkTokens({ h, s, accent }: Hue): SpaceTokens {
  return {
    shell: hsl(h, s, 14),
    glow: hsl(h, s, 18),
    card: hsl(h, s, 14),
    hover: hsl(h, s, 16),
    field: hsl(h, s, 12.5),
    hairline: hsl(h, s, 20),
    ink: hsl(h, s * 0.5, 92),
    inkMuted: hsl(h, s * 0.6, 66),
    inkFaint: hsl(h, s * 0.6, 46),
    tint: hsl(h, accent, 66),
    hi: 'rgba(255, 255, 255, 0.075)',
    lo: 'rgba(0, 0, 0, 0.7)',
  }
}

function lightTokens({ h, s, accent }: Hue): SpaceTokens {
  return {
    shell: hsl(h, s * 1.4, 89),
    glow: hsl(h, s * 1.4, 94),
    card: hsl(h, s * 1.4, 89),
    hover: hsl(h, s * 1.4, 86.5),
    field: hsl(h, s * 1.4, 91),
    hairline: hsl(h, s * 1.2, 80),
    ink: hsl(h, s * 2, 14),
    inkMuted: hsl(h, s * 1.6, 36),
    inkFaint: hsl(h, s * 1.4, 56),
    tint: hsl(h, accent, 40),
    hi: 'rgba(255, 255, 255, 0.9)',
    lo: rgba(h, Math.max(s * 2.5, 0.15), 38, 0.3),
  }
}

/** L'espace d'un profil : `id` est le profil, `hueId` sa teinte (par defaut la teinte de meme nom). */
export function buildSpace(id: string, scheme: Scheme, hueId?: SpaceId): Space {
  const hue = HUES.find((candidate) => candidate.id === (hueId ?? id)) ?? HUES[0]!
  return { id, hue: hue.id, name: hue.name, scheme, tokens: scheme === 'dark' ? darkTokens(hue) : lightTokens(hue) }
}

/** Jetons qui dependent du schema clair/sombre, pas de la teinte. */
export interface SchemeTokens {
  guard: string
  warn: string
  danger: string
}

export const SCHEME_TOKENS: Record<Scheme, SchemeTokens> = {
  dark: { guard: '#55b98d', warn: '#d2a056', danger: '#d5564c' },
  light: { guard: '#2f8f66', warn: '#a4712a', danger: '#b8423a' },
}

/** Les ombres du relief, construites sur la lumiere et l'ombre de l'espace. */
export function reliefShadows({ hi, lo }: Pick<SpaceTokens, 'hi' | 'lo'>): Record<'card' | 'lift' | 'pressed' | 'field', string> {
  return {
    card: `-3px -3px 7px ${hi}, 3px 3px 8px ${lo}`,
    lift: `-8px -8px 20px ${hi}, 10px 10px 24px ${lo}`,
    pressed: `inset -3px -3px 7px ${hi}, inset 3px 3px 8px ${lo}`,
    field: `inset -2px -2px 5px ${hi}, inset 2px 2px 6px ${lo}`,
  }
}

export function isSpaceId(value: string): value is SpaceId {
  return HUES.some((hue) => hue.id === value)
}

export function isScheme(value: string): value is Scheme {
  return value === 'dark' || value === 'light'
}
