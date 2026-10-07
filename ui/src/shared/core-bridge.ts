// Responsabilite : resoudre le pont vers le coeur — le vrai s'il est injecte, sinon le faux de developpement.

import type { CoreBridge, CoreEvent, UiRequest } from './contract'
import { createFakeCore, type FakeControls } from './fake/fake-core'

export interface BridgeHandle {
  bridge: CoreBridge
  /** Vrai quand le coeur Rust est absent et que le faux coeur prend le relais. */
  simulated: boolean
  /** Leviers de simulation, uniquement quand le faux coeur tourne. */
  fake: FakeControls | null
}

// Singleton assume : un seul pont par page, resolu au premier acces.
let handle: BridgeHandle | null = null

const WAIT_STEP_MS = 20

/**
 * Pont differe : servie par Echo (`echo://`), l'interface peut s'executer avant que le coeur ait injecte
 * son pont (relance qui rouvre beaucoup d'onglets d'un coup). Plutot que de basculer pour toujours dans
 * le faux coeur, on met les demandes en file et on se branche des que le vrai pont apparait.
 */
function deferredBridge(): CoreBridge {
  const queue: UiRequest[] = []
  const listeners: Array<(event: CoreEvent) => void> = []
  const detach: Array<() => void> = []
  let real: CoreBridge | null = null
  const attach = (bridge: CoreBridge): void => {
    real = bridge
    for (const listener of listeners) detach.push(bridge.subscribe(listener))
    for (const request of queue.splice(0)) bridge.send(request)
  }
  const poll = (): void => {
    if (window.echo !== undefined) attach(window.echo)
    else window.setTimeout(poll, WAIT_STEP_MS)
  }
  poll()
  return {
    send: (request) => (real === null ? queue.push(request) : real.send(request)),
    subscribe: (listener) => {
      listeners.push(listener)
      if (real !== null) detach.push(real.subscribe(listener))
      return () => {
        const index = listeners.indexOf(listener)
        if (index >= 0) listeners.splice(index, 1)
      }
    },
  }
}

export function resolveBridge(): BridgeHandle {
  if (handle !== null) return handle
  const injected = window.echo
  if (injected !== undefined) {
    handle = { bridge: injected, simulated: false, fake: null }
    return handle
  }
  if (window.location.protocol === 'echo:') {
    handle = { bridge: deferredBridge(), simulated: false, fake: null }
    return handle
  }
  const fake = createFakeCore()
  handle = { bridge: fake.bridge, simulated: true, fake: fake.controls }
  return handle
}
