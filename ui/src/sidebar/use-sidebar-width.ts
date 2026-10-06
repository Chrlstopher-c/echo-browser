// Responsabilite : repli de la barre. La largeur reclamee au coeur ne change jamais ; le repli
// retire seulement la place reservee a la barre, qui revient par-dessus la page au bord gauche.

import { useCallback, useEffect, useMemo, useState } from 'react'
import type { UiRequest } from '../shared/contract'
import { readLocal, writeLocal } from '../shared/local-store'
import { widthFor } from './sidebar-geometry'

const STORE_KEY = 'echo.sidebar.collapsed'

export interface SidebarWidth {
  collapsed: boolean
  width: number
  toggle: () => void
  expand: () => void
}

function readStored(): boolean {
  return readLocal(STORE_KEY, (raw) => (typeof raw === 'boolean' ? raw : null)) ?? false
}

export function useSidebarWidth(send: (request: UiRequest) => void): SidebarWidth {
  const [collapsed, setCollapsed] = useState<boolean>(readStored)

  useEffect(() => {
    send({ kind: 'setChromeWidth', pixels: widthFor() })
    send({ kind: 'setSidebarCollapsed', collapsed: readStored() })
  }, [send])

  const apply = useCallback(
    (next: boolean): void => {
      setCollapsed(next)
      writeLocal(STORE_KEY, next)
      send({ kind: 'setSidebarCollapsed', collapsed: next })
    },
    [send],
  )

  const toggle = useCallback((): void => apply(!collapsed), [apply, collapsed])
  const expand = useCallback((): void => {
    if (collapsed) apply(false)
  }, [apply, collapsed])
  return useMemo(() => ({ collapsed, width: widthFor(), toggle, expand }), [collapsed, toggle, expand])
}
