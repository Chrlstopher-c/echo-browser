// Responsabilite : feuille ouverte sur la liste d'onglets. Une seule a la fois ; Echap la ferme.

import { useCallback, useEffect, useMemo, useState } from 'react'
import type { SheetId } from './sheet'

export interface SheetController {
  current: SheetId | null
  toggle: (sheet: SheetId) => void
  open: (sheet: SheetId) => void
  close: () => void
}

export function useSheet(): SheetController {
  const [current, setCurrent] = useState<SheetId | null>(null)

  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === 'Escape' && !event.defaultPrevented) setCurrent(null)
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])

  const toggle = useCallback((sheet: SheetId): void => {
    setCurrent((open) => (open === sheet ? null : sheet))
  }, [])
  const open = useCallback((sheet: SheetId): void => setCurrent(sheet), [])
  const close = useCallback((): void => setCurrent(null), [])

  return useMemo(() => ({ current, toggle, open, close }), [current, toggle, open, close])
}
