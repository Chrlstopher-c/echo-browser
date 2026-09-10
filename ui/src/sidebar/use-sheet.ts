// Responsabilite : pile des feuilles ouvertes. Une seule visible ; retour possible vers la precedente.

import { useCallback, useEffect, useState } from 'react'
import type { SheetId } from './sheet'

export interface SheetController {
  current: SheetId | null
  /** Vrai quand une feuille precedente attend derriere la courante. */
  canGoBack: boolean
  toggle: (sheet: SheetId) => void
  push: (sheet: SheetId) => void
  back: () => void
  close: () => void
}

export function useSheet(): SheetController {
  const [stack, setStack] = useState<SheetId[]>([])

  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') setStack([])
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])

  const toggle = useCallback((sheet: SheetId): void => {
    setStack((current) => (current.at(-1) === sheet ? [] : [sheet]))
  }, [])
  const push = useCallback((sheet: SheetId): void => setStack((current) => [...current, sheet]), [])
  const back = useCallback((): void => setStack((current) => current.slice(0, -1)), [])
  const close = useCallback((): void => setStack([]), [])

  return { current: stack.at(-1) ?? null, canGoBack: stack.length > 1, toggle, push, back, close }
}
