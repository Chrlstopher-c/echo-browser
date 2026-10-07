// Responsabilite : lire ce que le coeur a mis dans le fragment de l'adresse du menu.
// Le contenu voyage la plutot que par un message : la page le lit des sa premiere ligne,
// sans un aller-retour pendant lequel le menu s'afficherait vide.

import type { ContextTarget, OverlayTheme } from '../shared/contract'
import { reliefShadows } from '../spaces/space-palette'

export interface MenuPayload {
  target: ContextTarget
  theme: OverlayTheme
}

const EMPTY: MenuPayload = {
  target: { entries: [], link: '', selection: '' },
  theme: {
    shell: '#212226', card: '#212226', hover: '#25262b', hairline: '#313339',
    ink: '#ebeced', inkMuted: '#a3a6ae', inkFaint: '#6c707a', hi: 'rgba(255, 255, 255, 0.075)',
    lo: 'rgba(0, 0, 0, 0.7)', tint: '#8f96a3', danger: '#e5484d',
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
  root.style.setProperty('--color-tint', theme.tint)
  // Liseres du relief : le menu ne peut pas deborder de sa surimpression, son relief se lit sur ses bords.
  root.style.setProperty('--color-hi', theme.hi)
  root.style.setProperty('--color-lo', theme.lo)
  const shadows = reliefShadows(theme)
  root.style.setProperty('--shadow-field', shadows.field)
  root.style.setProperty('--shadow-pressed', shadows.pressed)
}
