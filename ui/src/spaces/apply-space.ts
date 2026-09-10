// Responsabilite : poser les jetons d'un espace sur le document. Les utilitaires Tailwind lisent
// `var(--color-*)`, une surcharge en ligne sur <html> suffit a reteinter toute l'interface.

import { SCHEME_TOKENS, type Space } from './space-palette'

const TOKEN_VARIABLE: Record<keyof Space['tokens'], string> = {
  shell: '--color-shell',
  glow: '--color-glow',
  card: '--color-card',
  hover: '--color-hover',
  field: '--color-field',
  hairline: '--color-hairline',
  ink: '--color-ink',
  inkMuted: '--color-ink-muted',
  inkFaint: '--color-ink-faint',
}

export function applySpace(space: Space): void {
  const root = document.documentElement
  const scheme = SCHEME_TOKENS[space.scheme]
  for (const [token, variable] of Object.entries(TOKEN_VARIABLE)) {
    // Justification : Object.entries perd le type des cles, la table ci-dessus les garantit.
    root.style.setProperty(variable, space.tokens[token as keyof Space['tokens']])
  }
  root.style.setProperty('--color-guard', scheme.guard)
  root.style.setProperty('--color-warn', scheme.warn)
  root.style.setProperty('--color-danger', scheme.danger)
  root.style.setProperty('--shadow-card', scheme.shadowCard)
  root.style.setProperty('--shadow-frame', scheme.shadowFrame)
  root.style.colorScheme = space.scheme
  root.dataset['space'] = space.id
}
