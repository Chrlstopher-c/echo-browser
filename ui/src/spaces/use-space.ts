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
  HUES,
  isScheme,
  isSpaceId,
  SCHEME_TOKENS,
  type Scheme,
  type Space,
  type SpaceId,
} from './space-palette'

const SPACE_KEY = 'echo.space'
const SCHEME_KEY = 'echo.scheme'

export interface SpaceController {
  space: Space
  select: (id: SpaceId) => void
  /** Passe a l'espace suivant ou precedent, en boucle. */
  cycle: (direction: 1 | -1) => void
  toggleScheme: () => void
}

/** Les couleurs dont une surimpression a besoin pour se fondre dans l'espace. */
function overlayTheme(space: Space): OverlayTheme {
  const { shell, card, hover, hairline, ink, inkMuted, inkFaint, hi, lo, tint } = space.tokens
  return { shell, card, hover, hairline, ink, inkMuted, inkFaint, hi, lo, tint, danger: SCHEME_TOKENS[space.scheme].danger }
}

function readStoredSpace(): SpaceId {
  return readLocal(SPACE_KEY, (raw) => (typeof raw === 'string' && isSpaceId(raw) ? raw : null)) ?? DEFAULT_SPACE
}

function readStoredScheme(): Scheme {
  return readLocal(SCHEME_KEY, (raw) => (typeof raw === 'string' && isScheme(raw) ? raw : null)) ?? DEFAULT_SCHEME
}

export function useSpace(send: (request: UiRequest) => void): SpaceController {
  const [id, setId] = useState<SpaceId>(readStoredSpace)
  const [scheme, setScheme] = useState<Scheme>(readStoredScheme)
  const space = useMemo(() => buildSpace(id, scheme), [id, scheme])

  useEffect(() => {
    applySpace(space)
    send({ kind: 'setAccent', color: space.tokens.shell })
    send({ kind: 'setColorScheme', dark: space.scheme === 'dark' })
    // Ce qui s'affiche au-dessus de la page est une page a part : elle ne partage pas
    // nos jetons, le coeur les lui transmet.
    send({ kind: 'setOverlayTheme', theme: overlayTheme(space) })
  }, [space, send])

  const select = useCallback((next: SpaceId): void => {
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
      const index = HUES.findIndex((candidate) => candidate.id === id)
      const next = HUES[(index + direction + HUES.length) % HUES.length]
      if (next !== undefined) select(next.id)
    },
    [id, select],
  )

  return { space, select, cycle, toggleScheme }
}
