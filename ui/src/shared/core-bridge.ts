// Responsabilite : resoudre le pont vers le coeur — le vrai s'il est injecte, sinon le faux de developpement.

import type { CoreBridge } from './contract'
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

export function resolveBridge(): BridgeHandle {
  if (handle !== null) return handle
  const injected = window.echo
  if (injected !== undefined) {
    handle = { bridge: injected, simulated: false, fake: null }
    return handle
  }
  const fake = createFakeCore()
  handle = { bridge: fake.bridge, simulated: true, fake: fake.controls }
  return handle
}
