// Responsabilite : lire ce que le coeur a mis dans le fragment de l'adresse du menu.
// Le contenu voyage la plutot que par un message : la page le lit des sa premiere ligne,
// sans un aller-retour pendant lequel le menu s'afficherait vide.

import type { ContextTarget, OverlayTheme } from '../shared/contract'

export interface MenuPayload {
  target: ContextTarget
  theme: OverlayTheme
}

const EMPTY: MenuPayload = {
  target: { entries: [], link: '', selection: '' },
  theme: {
    shell: '#141517', card: '#24272c', hover: '#1e2126', hairline: '#2a2d33',
    ink: '#e8e6e3', inkMuted: '#9b9a97', inkFaint: '#6b6a68', danger: '#e5484d',
  },
}

export function readPayload(hash: string): MenuPayload {
  const raw = hash.startsWith('#') ? hash.slice(1) : hash
  if (raw === '') return EMPTY
  try {
    const parsed: unknown = JSON.parse(decodeURIComponent(raw))
    if (typeof parsed !== 'object' || parsed === null) return EMPTY
    const payload = parsed as Partial<MenuPayload>
    return {
      target: payload.target ?? EMPTY.target,
      theme: payload.theme ?? EMPTY.theme,
    }
  } catch (error) {
    console.error('menu illisible', error)
    return EMPTY
  }
}

/** Pose les couleurs de l'espace sur le document, comme la barre le fait pour le sien. */
export function applyTheme(theme: OverlayTheme): void {
  const root = document.documentElement
  root.style.setProperty('--color-shell', theme.shell)
  root.style.setProperty('--color-card', theme.card)
  root.style.setProperty('--color-hover', theme.hover)
  root.style.setProperty('--color-hairline', theme.hairline)
  root.style.setProperty('--color-ink', theme.ink)
  root.style.setProperty('--color-ink-muted', theme.inkMuted)
  root.style.setProperty('--color-ink-faint', theme.inkFaint)
  root.style.setProperty('--color-danger', theme.danger)
}
