// Responsabilite : repli de la barre et largeur reclamee au coeur. Au repli, le coeur est
// prevenu apres l'animation (la barre retrecit dans sa fenetre) ; au depli, avant (la fenetre
// s'elargit puis la barre s'y deploie). La page ne chevauche jamais la barre.

import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { UiRequest } from '../shared/contract'
import { readLocal, writeLocal } from '../shared/local-store'
import { widthFor } from './sidebar-geometry'

const STORE_KEY = 'echo.sidebar.collapsed'

export interface SidebarWidth {
  collapsed: boolean
  width: number
  toggle: () => void
  expand: () => void
  /** A appeler quand l'animation de largeur se termine. */
  onSettled: () => void
}

function readStored(): boolean {
  return readLocal(STORE_KEY, (raw) => (typeof raw === 'boolean' ? raw : null)) ?? false
}

export function useSidebarWidth(send: (request: UiRequest) => void): SidebarWidth {
  const [collapsed, setCollapsed] = useState<boolean>(readStored)
  const pendingClaim = useRef<number | null>(null)

  useEffect(() => {
    send({ kind: 'setChromeWidth', pixels: widthFor(readStored()) })
  }, [send])

  const apply = useCallback(
    (next: boolean): void => {
      setCollapsed(next)
      writeLocal(STORE_KEY, next)
      send({ kind: 'setSidebarCollapsed', collapsed: next })
      const pixels = widthFor(next)
      pendingClaim.current = next ? pixels : null
      if (!next) send({ kind: 'setChromeWidth', pixels })
    },
    [send],
  )

  const onSettled = useCallback((): void => {
    if (pendingClaim.current === null) return
    send({ kind: 'setChromeWidth', pixels: pendingClaim.current })
    pendingClaim.current = null
  }, [send])

  return useGestures(collapsed, apply, onSettled)
}

function useGestures(collapsed: boolean, apply: (next: boolean) => void, onSettled: () => void): SidebarWidth {
  const toggle = useCallback((): void => apply(!collapsed), [apply, collapsed])
  const expand = useCallback((): void => {
    if (collapsed) apply(false)
  }, [apply, collapsed])
  return useMemo(
    () => ({ collapsed, width: widthFor(collapsed), toggle, expand, onSettled }),
    [collapsed, toggle, expand, onSettled],
  )
}
