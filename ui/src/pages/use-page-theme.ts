// Responsabilite : la page prend la teinte et le schema choisis dans la barre (stockage local partage),
// sans jamais les modifier : changer de teinte ici changerait de profil.

import { useEffect } from 'react'
import { applySpace } from '../spaces/apply-space'
import { buildSpace } from '../spaces/space-palette'
import { readStoredScheme, readStoredSpace } from '../spaces/use-space'

export function usePageTheme(): void {
  useEffect(() => {
    const apply = (): void => applySpace(buildSpace(readStoredSpace(), readStoredScheme()))
    apply()
    window.addEventListener('storage', apply)
    return () => window.removeEventListener('storage', apply)
  }, [])
}
