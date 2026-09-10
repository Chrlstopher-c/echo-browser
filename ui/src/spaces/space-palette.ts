// Responsabilite : les espaces — chaque teinte est un jeu complet de jetons, source unique de la
// palette. Le systeme de design (theme.css) ne porte que les noms et l'espace par defaut.

export type SpaceId = 'graphite' | 'sable' | 'rose' | 'foret' | 'ardoise' | 'lin'

export type Scheme = 'dark' | 'light'

export interface SpaceTokens {
  /** Couleur plate de la barre et du cadre autour de la page. */
  shell: string
  /** Lueur en haut a gauche de la barre. */
  glow: string
  /** Carte posee : onglet actif, pastille active. */
  card: string
  hover: string
  field: string
  hairline: string
  ink: string
  inkMuted: string
  inkFaint: string
  /** Teinte propre a l'espace : pastille du selecteur, liseré de l'onglet actif. */
  tint: string
}

export interface Space {
  id: SpaceId
  name: string
  scheme: Scheme
  tokens: SpaceTokens
}

export const DEFAULT_SPACE: SpaceId = 'graphite'

export const SPACES: readonly Space[] = [
  {
    id: 'graphite',
    name: 'Graphite',
    scheme: 'dark',
    tokens: {
      shell: '#141517', glow: '#23262b', card: '#24272c', hover: '#1e2126', field: '#1b1d21',
      hairline: '#2a2d33', ink: '#e7e8ea', inkMuted: '#9a9fa7', inkFaint: '#63686f', tint: '#8f96a3',
    },
  },
  {
    id: 'sable',
    name: 'Sable',
    scheme: 'dark',
    tokens: {
      shell: '#1b1712', glow: '#3d2f1d', card: '#2d2519', hover: '#251f16', field: '#221c15',
      hairline: '#362c20', ink: '#ece4d6', inkMuted: '#a89b86', inkFaint: '#6f6555', tint: '#d0a468',
    },
  },
  {
    id: 'rose',
    name: 'Rose',
    scheme: 'dark',
    tokens: {
      shell: '#1b1417', glow: '#3e2230', card: '#2e1f26', hover: '#26191e', field: '#221820',
      hairline: '#382630', ink: '#eee0e4', inkMuted: '#ab949b', inkFaint: '#705d64', tint: '#d4849c',
    },
  },
  {
    id: 'foret',
    name: 'Forêt',
    scheme: 'dark',
    tokens: {
      shell: '#121916', glow: '#1f3629', card: '#1e2c25', hover: '#18241e', field: '#16201b',
      hairline: '#243329', ink: '#e1ebe4', inkMuted: '#93a89a', inkFaint: '#5e7066', tint: '#6fbf93',
    },
  },
  {
    id: 'ardoise',
    name: 'Ardoise',
    scheme: 'dark',
    tokens: {
      shell: '#13161c', glow: '#212c3e', card: '#212834', hover: '#1b212b', field: '#181d26',
      hairline: '#28303d', ink: '#e3e7ee', inkMuted: '#969eac', inkFaint: '#626a78', tint: '#7ea2dc',
    },
  },
  {
    id: 'lin',
    name: 'Lin',
    scheme: 'light',
    tokens: {
      shell: '#ebe5d9', glow: '#faf6ee', card: '#fbf9f4', hover: '#e1dacb', field: '#f5f0e7',
      hairline: '#d6cebf', ink: '#2a2620', inkMuted: '#6f675b', inkFaint: '#a39a8b', tint: '#b8905a',
    },
  },
]

/** Jetons qui dependent du schema clair/sombre, pas de la teinte. */
export interface SchemeTokens {
  guard: string
  warn: string
  danger: string
  shadowCard: string
  shadowLift: string
  shadowFrame: string
}

export const SCHEME_TOKENS: Record<Scheme, SchemeTokens> = {
  dark: {
    guard: '#55b98d',
    warn: '#d2a056',
    danger: '#d5564c',
    shadowCard: '0 1px 2px rgb(0 0 0 / 0.35), 0 0 0 1px rgb(255 255 255 / 0.05)',
    shadowLift: '0 8px 24px -8px rgb(0 0 0 / 0.6), 0 0 0 1px rgb(255 255 255 / 0.07)',
    shadowFrame: '0 16px 48px -16px rgb(0 0 0 / 0.55), 0 0 0 1px rgb(0 0 0 / 0.25)',
  },
  light: {
    guard: '#2f8f66',
    warn: '#a4712a',
    danger: '#b8423a',
    shadowCard: '0 1px 2px rgb(0 0 0 / 0.08), 0 0 0 1px rgb(0 0 0 / 0.04)',
    shadowLift: '0 8px 24px -8px rgb(40 30 10 / 0.25), 0 0 0 1px rgb(0 0 0 / 0.05)',
    shadowFrame: '0 16px 48px -20px rgb(40 30 10 / 0.35), 0 0 0 1px rgb(0 0 0 / 0.06)',
  },
}

export function spaceOf(id: SpaceId): Space {
  return SPACES.find((space) => space.id === id) ?? SPACES[0]!
}

export function isSpaceId(value: string): value is SpaceId {
  return SPACES.some((space) => space.id === value)
}
