// Responsabilite : la barre repliee se cache quand la souris la quitte, apres un court delai
// (le trajet vers un bouton en bord de fenetre ne doit pas la fermer).

import { useCallback, useEffect, useMemo, useRef } from 'react'
import type { UiRequest } from '../shared/contract'

const HIDE_DELAY_MS = 280

export interface RevealOnHover {
  enter: () => void
  leave: () => void
}

export function useRevealOnHover(send: (request: UiRequest) => void, collapsed: boolean): RevealOnHover {
  const timer = useRef(0)
  const cancel = useCallback((): void => window.clearTimeout(timer.current), [])
  const enter = useCallback((): void => cancel(), [cancel])
  const leave = useCallback((): void => {
    if (!collapsed) return
    cancel()
    timer.current = window.setTimeout(() => send({ kind: 'revealSidebar', reveal: false }), HIDE_DELAY_MS)
  }, [cancel, collapsed, send])
  useEffect(() => cancel, [cancel])
  return useMemo(() => ({ enter, leave }), [enter, leave])
}
