// Responsabilite : poser les jetons d'un espace sur le document. Les utilitaires Tailwind lisent
// `var(--color-*)`, une surcharge en ligne sur <html> suffit a reteinter toute l'interface.

import { SCHEME_TOKENS, type Space, type SpaceTokens } from './space-palette'

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
}

const TOKEN_KEYS = Object.keys(TOKEN_VARIABLE) as Array<keyof SpaceTokens> // Justification : cles de la table.

export function applySpace(space: Space): void {
  const root = document.documentElement
  const scheme = SCHEME_TOKENS[space.scheme]
  for (const token of TOKEN_KEYS) root.style.setProperty(TOKEN_VARIABLE[token], space.tokens[token])
  root.style.setProperty('--color-guard', scheme.guard)
  root.style.setProperty('--color-warn', scheme.warn)
  root.style.setProperty('--color-danger', scheme.danger)
  root.style.setProperty('--shadow-card', scheme.shadowCard)
  root.style.setProperty('--shadow-lift', scheme.shadowLift)
  root.style.setProperty('--shadow-frame', scheme.shadowFrame)
  root.style.colorScheme = space.scheme
  root.dataset['space'] = space.id
}
