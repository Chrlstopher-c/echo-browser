// Responsabilite : resoudre le pont vers le coeur — le vrai s'il est injecte, sinon le faux de developpement.

import type { CoreBridge } from './contract'
import { createFakeCore } from './fake-core'

export interface BridgeHandle {
  bridge: CoreBridge
  /** Vrai quand le coeur Rust est absent et que le faux coeur prend le relais. */
  simulated: boolean
}

// Singleton assume : un seul pont par page, resolu au premier acces.
let handle: BridgeHandle | null = null

export function resolveBridge(): BridgeHandle {
  if (handle !== null) return handle
  const injected = window.echo
  handle =
    injected !== undefined
      ? { bridge: injected, simulated: false }
      : { bridge: createFakeCore(), simulated: true }
  return handle
}
