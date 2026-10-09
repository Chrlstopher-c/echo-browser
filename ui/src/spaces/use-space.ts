// Responsabilite : espace courant (teinte + schema clair/sombre) — choix persistes, jetons poses sur le
// document, couleurs envoyees au coeur pour le cadre autour de la page et les surimpressions.

import { useCallback, useEffect, useMemo, useState } from 'react'
import type { OverlayTheme, UiRequest } from '../shared/contract'
import { readLocal, writeLocal } from '../shared/local-store'
import { applySpace, themeObject } from './apply-space'
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
const FOLLOW_KEY = 'echo.scheme.system'

/** Choix du theme : fixe, ou celui du bureau. */
export type SchemeChoice = Scheme | 'system'

export interface SpaceController {
  space: Space
  /** Passe au profil `id`. */
  select: (id: string) => void
  /** Passe a l'espace suivant ou precedent, en boucle. */
  cycle: (direction: 1 | -1) => void
  toggleScheme: () => void
  /** Choix affiche dans les Reglages. */
  schemeChoice: SchemeChoice
  setSchemeChoice: (choice: SchemeChoice) => void
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
function usePublishSpace(space: Space, send: (request: UiRequest) => void, choice: SchemeChoice): void {
  useEffect(() => {
    applySpace(space)
    send({ kind: 'setAccent', color: space.tokens.shell })
    send({ kind: 'setColorScheme', dark: space.scheme === 'dark' })
    send({ kind: 'setSpace', id: space.id })
    // Ce qui s'affiche au-dessus de la page est une page a part : elle ne partage pas
    // nos jetons, le coeur les lui transmet.
    send({ kind: 'setOverlayTheme', theme: overlayTheme(space) })
    send({ kind: 'setPageTheme', theme: { ...themeObject(space), choice } })
  }, [space, send, choice])
}

const NOW_KEY = 'echo.scheme.now'

function readFollow(): boolean {
  return readLocal(FOLLOW_KEY, (raw) => (typeof raw === 'boolean' ? raw : null)) ?? false
}

/** Choix enregistre (pour les pages pleine largeur, qui partagent le stockage de la barre). */
export function readSchemeChoice(): SchemeChoice {
  return readFollow() ? 'system' : readStoredScheme()
}

export function writeSchemeChoice(choice: SchemeChoice): void {
  writeLocal(FOLLOW_KEY, choice === 'system')
  if (choice !== 'system') writeLocal(SCHEME_KEY, choice)
}

/** Schema reellement affiche par la barre (« Système » resolu). */
export function readSchemeNow(): Scheme {
  return readLocal(NOW_KEY, (raw) => (typeof raw === 'string' && isScheme(raw) ? raw : null)) ?? readStoredScheme()
}

export function useSpace(
  send: (request: UiRequest) => void, profiles: ProfileNames, systemDark: boolean | null,
): SpaceController {
  const [stored, setId] = useState<string>(readStoredSpace)
  const [fixed, setScheme] = useState<Scheme>(readStoredScheme)
  const [follow, setFollow] = useState<boolean>(readFollow)
  const scheme: Scheme = follow ? (systemDark === null ? fixed : systemDark ? 'dark' : 'light') : fixed
  // Un profil supprime (ici ou sur une autre machine) : retour au profil principal.
  const id = profiles.list.some((p) => p.id === stored) ? stored : DEFAULT_SPACE
  const hue = profiles.hueOf(id)
  const space = useMemo(() => buildSpace(id, scheme, hue), [id, scheme, hue])

  usePublishSpace(space, send, follow ? 'system' : fixed)
  useEffect(() => writeLocal(NOW_KEY, scheme), [scheme])
  // Le choix fait dans la page Reglages (autre document) arrive par le stockage partage.
  useEffect(() => {
    const reload = (): void => {
      setScheme(readStoredScheme())
      setFollow(readFollow())
    }
    window.addEventListener('storage', reload)
    return () => window.removeEventListener('storage', reload)
  }, [])

  const select = useCallback((next: string): void => {
    setId(next)
    writeLocal(SPACE_KEY, next)
  }, [])

  const setSchemeChoice = useCallback((choice: SchemeChoice): void => {
    writeSchemeChoice(choice)
    setFollow(choice === 'system')
    if (choice !== 'system') setScheme(choice)
  }, [])

  // La bascule soleil/lune fixe le theme oppose a celui affiche : elle quitte le mode « Système ».
  const toggleScheme = useCallback((): void => {
    setSchemeChoice(scheme === 'dark' ? 'light' : 'dark')
  }, [scheme, setSchemeChoice])

  const cycle = useCallback(
    (direction: 1 | -1): void => {
      const list = profiles.list
      const index = list.findIndex((candidate) => candidate.id === id)
      const next = list[(index + direction + list.length) % list.length]
      if (next !== undefined) select(next.id)
    },
    [id, select, profiles.list],
  )

  return { space, select, cycle, toggleScheme, schemeChoice: follow ? 'system' : fixed, setSchemeChoice }
}
