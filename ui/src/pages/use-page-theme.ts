// Responsabilite : la page prend la teinte et le schema choisis dans la barre, sans jamais les modifier (changer de
// teinte ici changerait de profil). Dans le profil principal, le stockage local partage avec la barre suffit ; dans un
// autre profil ou en navigation privee, ce stockage est vide : le theme vient du coeur (au chargement, puis a chaque
// changement).

import { useEffect, useState } from 'react'
import { applySpace, applyThemeObject } from '../spaces/apply-space'
import { buildSpace } from '../spaces/space-palette'
import { readLocal } from '../shared/local-store'
import { readSchemeNow, readStoredSpace } from '../spaces/use-space'

const SHARED_KEY = 'echo.scheme.now'

/** Rend le theme publie par la barre (pour les Reglages), demande au coeur au chargement. */
export function usePageTheme(fromCore: unknown): unknown {
  const [fetched, setFetched] = useState<unknown>(null)
  useEffect(() => {
    fetch('echo://ui/data/theme')
      .then((reply) => reply.json())
      .then(setFetched)
      .catch((error: unknown) => console.warn('theme des pages indisponible', error))
  }, [])
  useEffect(() => {
    const shared = readLocal(SHARED_KEY, (raw) => (typeof raw === 'string' ? raw : null)) !== null
    const apply = (): void => applySpace(buildSpace(readStoredSpace(), readSchemeNow()))
    if (shared) {
      apply()
      window.addEventListener('storage', apply)
      return () => window.removeEventListener('storage', apply)
    }
    return undefined
  }, [])
  const theme = fromCore ?? fetched
  useEffect(() => {
    if (theme !== null) applyThemeObject(theme)
  }, [theme])
  return theme
}
