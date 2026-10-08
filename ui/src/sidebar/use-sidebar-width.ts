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

/**
 * `narrow` : fenetre etroite. La barre s'y replie d'elle-meme sans toucher au choix enregistre, qui revient quand la
 * fenetre s'elargit ; un depli manuel en fenetre etroite ne vaut que pour ce passage.
 */
export function useSidebarWidth(send: (request: UiRequest) => void, narrow: boolean): SidebarWidth {
  const [stored, setStored] = useState<boolean>(readStored)
  const [narrowCollapsed, setNarrowCollapsed] = useState(true)
  const collapsed = narrow ? narrowCollapsed : stored

  useEffect(() => {
    send({ kind: 'setChromeWidth', pixels: widthFor() })
  }, [send])
  useEffect(() => {
    if (narrow) setNarrowCollapsed(true)
  }, [narrow])
  useEffect(() => {
    send({ kind: 'setSidebarCollapsed', collapsed })
  }, [send, collapsed])

  const apply = useCallback(
    (next: boolean): void => {
      if (narrow) {
        setNarrowCollapsed(next)
        return
      }
      setStored(next)
      writeLocal(STORE_KEY, next)
    },
    [narrow],
  )

  const toggle = useCallback((): void => apply(!collapsed), [apply, collapsed])
  const expand = useCallback((): void => {
    if (collapsed) apply(false)
  }, [apply, collapsed])
  return useMemo(() => ({ collapsed, width: widthFor(), toggle, expand }), [collapsed, toggle, expand])
}
