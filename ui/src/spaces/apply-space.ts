// Responsabilite : poser les jetons d'un espace sur le document. Les utilitaires Tailwind lisent
// `var(--color-*)`, une surcharge en ligne sur <html> suffit a reteinter toute l'interface. Les pages
// statiques (accueil, terminal, liseré) lisent les memes jetons dans le stockage local.

import { writeLocal } from '../shared/local-store'
import { reliefShadows, SCHEME_TOKENS, type Space, type SpaceTokens } from './space-palette'

export const THEME_STORE_KEY = 'echo.theme'

const TOKEN_VARIABLE: Record<keyof SpaceTokens, string> = {
  shell: '--color-shell',
  glow: '--color-glow',
  card: '--color-card',
  hover: '--color-hover',
  field: '--color-field',
  hairline: '--color-hairline',
  ink: '--color-ink',
  inkMuted: '--color-ink-muted',
  inkFaint: '--color-ink-faint',
  tint: '--color-tint',
  hi: '--hi',
  lo: '--lo',
}

const TOKEN_KEYS = Object.keys(TOKEN_VARIABLE) as Array<keyof SpaceTokens> // Justification : cles de la table.

export function applySpace(space: Space): void {
  const root = document.documentElement
  const scheme = SCHEME_TOKENS[space.scheme]
  const shadows = reliefShadows(space.tokens)
  for (const token of TOKEN_KEYS) root.style.setProperty(TOKEN_VARIABLE[token], space.tokens[token])
  root.style.setProperty('--color-guard', scheme.guard)
  root.style.setProperty('--color-warn', scheme.warn)
  root.style.setProperty('--color-danger', scheme.danger)
  root.style.setProperty('--shadow-card', shadows.card)
  root.style.setProperty('--shadow-lift', shadows.lift)
  root.style.setProperty('--shadow-pressed', shadows.pressed)
  root.style.setProperty('--shadow-field', shadows.field)
  root.style.colorScheme = space.scheme
  root.dataset['space'] = space.id
  root.dataset['scheme'] = space.scheme
  writeLocal(THEME_STORE_KEY, { scheme: space.scheme, ...space.tokens, ...scheme, shadows })
}
