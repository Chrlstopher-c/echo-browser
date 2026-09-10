// Responsabilite : espace courant — choix persiste, jetons poses sur le document, teinte envoyee
// au coeur pour le cadre autour de la page.

import { useCallback, useEffect, useState } from 'react'
import type { OverlayTheme, UiRequest } from '../shared/contract'
import { readLocal, writeLocal } from '../shared/local-store'
import { applySpace } from './apply-space'
import {
  DEFAULT_SPACE,
  isSpaceId,
  SCHEME_TOKENS,
  SPACES,
  spaceOf,
  type Space,
  type SpaceId,
} from './space-palette'

const STORE_KEY = 'echo.space'

export interface SpaceController {
  space: Space
  select: (id: SpaceId) => void
  /** Passe a l'espace suivant ou precedent, en boucle. */
  cycle: (direction: 1 | -1) => void
}

/** Les quelques couleurs dont une surimpression a besoin pour se fondre dans l'espace. */
function overlayTheme(space: Space): OverlayTheme {
  const { shell, card, hover, hairline, ink, inkMuted, inkFaint } = space.tokens
  return { shell, card, hover, hairline, ink, inkMuted, inkFaint, danger: SCHEME_TOKENS[space.scheme].danger }
}

function readStoredSpace(): SpaceId {
  return readLocal(STORE_KEY, (raw) => (typeof raw === 'string' && isSpaceId(raw) ? raw : null)) ?? DEFAULT_SPACE
}

export function useSpace(send: (request: UiRequest) => void): SpaceController {
  const [id, setId] = useState<SpaceId>(readStoredSpace)
  const space = spaceOf(id)

  useEffect(() => {
    applySpace(space)
    send({ kind: 'setAccent', color: space.tokens.shell })
    // Ce qui s'affiche au-dessus de la page est une page a part : elle ne partage pas
    // nos jetons, le coeur les lui transmet.
    send({ kind: 'setOverlayTheme', theme: overlayTheme(space) })
  }, [space, send])

  const select = useCallback((next: SpaceId): void => {
    setId(next)
    writeLocal(STORE_KEY, next)
  }, [])

  const cycle = useCallback(
    (direction: 1 | -1): void => {
      const index = SPACES.findIndex((candidate) => candidate.id === id)
      const next = SPACES[(index + direction + SPACES.length) % SPACES.length]
      if (next !== undefined) select(next.id)
    },
    [id, select],
  )

  return { space, select, cycle }
}
