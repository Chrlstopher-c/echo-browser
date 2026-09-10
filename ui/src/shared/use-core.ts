// Responsabilite : brancher React sur le coeur — abonnement aux evenements, envoi des requetes.

import { useCallback, useEffect, useMemo, useReducer } from 'react'
import type { ShieldView, TabView, UiRequest } from './contract'
import { resolveBridge } from './core-bridge'
import { activeShieldOf, activeTabOf, EMPTY_CORE_STATE, reduceCore, type CoreState, type Notice } from './core-state'

export interface CoreConnection {
  state: CoreState
  activeTab: TabView | null
  shield: ShieldView
  notice: Notice | null
  simulated: boolean
  send: (request: UiRequest) => void
}

export function useCore(): CoreConnection {
  const { bridge, simulated } = useMemo(() => resolveBridge(), [])
  const [state, dispatch] = useReducer(reduceCore, EMPTY_CORE_STATE)

  useEffect(() => bridge.subscribe(dispatch), [bridge])

  const send = useCallback(
    (request: UiRequest): void => {
      bridge.send(request)
    },
    [bridge],
  )

  const activeTab = useMemo(() => activeTabOf(state), [state])
  const shield = useMemo(() => activeShieldOf(state), [state])

  return { state, activeTab, shield, notice: state.notice, simulated, send }
}
