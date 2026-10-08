// Responsabilite : espace courant (teinte + schema clair/sombre) — choix persistes, jetons poses sur le
// document, couleurs envoyees au coeur pour le cadre autour de la page et les surimpressions.

import { useCallback, useEffect, useMemo, useState } from 'react'
import type { OverlayTheme, UiRequest } from '../shared/contract'
import { readLocal, writeLocal } from '../shared/local-store'
import { applySpace } from './apply-space'
import {
  buildSpace,
  DEFAULT_SCHEME,
  DEFAULT_SPACE,
  isScheme,
  SCHEME_TOKENS,
  type Scheme,
  type Space,
} from './space-palette'
import type { ProfileNames } from './use-profile-names'

const SPACE_KEY = 'echo.space'
const SCHEME_KEY = 'echo.scheme'

export interface SpaceController {
  space: Space
  /** Passe au profil `id`. */
  select: (id: string) => void
  /** Passe a l'espace suivant ou precedent, en boucle. */
  cycle: (direction: 1 | -1) => void
  toggleScheme: () => void
}

/** Les couleurs dont une surimpression a besoin pour se fondre dans l'espace. */
function overlayTheme(space: Space): OverlayTheme {
  const { shell, card, hover, hairline, ink, inkMuted, inkFaint, hi, lo, tint } = space.tokens
  return { shell, card, hover, hairline, ink, inkMuted, inkFaint, hi, lo, tint, danger: SCHEME_TOKENS[space.scheme].danger }
}

export function readStoredSpace(): string {
  return readLocal(SPACE_KEY, (raw) => (typeof raw === 'string' && /^[A-Za-z0-9_-]{1,40}$/.test(raw) ? raw : null))
    ?? DEFAULT_SPACE
}

export function readStoredScheme(): Scheme {
  return readLocal(SCHEME_KEY, (raw) => (typeof raw === 'string' && isScheme(raw) ? raw : null)) ?? DEFAULT_SCHEME
}

/** Applique l'espace a l'interface et le transmet au coeur (cadre, pages, surimpressions). */
function usePublishSpace(space: Space, send: (request: UiRequest) => void): void {
  useEffect(() => {
    applySpace(space)
    send({ kind: 'setAccent', color: space.tokens.shell })
    send({ kind: 'setColorScheme', dark: space.scheme === 'dark' })
    send({ kind: 'setSpace', id: space.id })
    // Ce qui s'affiche au-dessus de la page est une page a part : elle ne partage pas
    // nos jetons, le coeur les lui transmet.
    send({ kind: 'setOverlayTheme', theme: overlayTheme(space) })
  }, [space, send])
}

export function useSpace(send: (request: UiRequest) => void, profiles: ProfileNames): SpaceController {
  const [stored, setId] = useState<string>(readStoredSpace)
  const [scheme, setScheme] = useState<Scheme>(readStoredScheme)
  // Un profil supprime (ici ou sur une autre machine) : retour au profil principal.
  const id = profiles.list.some((p) => p.id === stored) ? stored : DEFAULT_SPACE
  const hue = profiles.hueOf(id)
  const space = useMemo(() => buildSpace(id, scheme, hue), [id, scheme, hue])

  usePublishSpace(space, send)

  const select = useCallback((next: string): void => {
    setId(next)
    writeLocal(SPACE_KEY, next)
  }, [])

  const toggleScheme = useCallback((): void => {
    setScheme((current) => {
      const next: Scheme = current === 'dark' ? 'light' : 'dark'
      writeLocal(SCHEME_KEY, next)
      return next
    })
  }, [])

  const cycle = useCallback(
    (direction: 1 | -1): void => {
      const list = profiles.list
      const index = list.findIndex((candidate) => candidate.id === id)
      const next = list[(index + direction + list.length) % list.length]
      if (next !== undefined) select(next.id)
    },
    [id, select, profiles.list],
  )

  return { space, select, cycle, toggleScheme }
}
