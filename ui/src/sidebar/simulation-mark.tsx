// Responsabilite : marqueur visible quand le faux coeur de developpement alimente l'interface.

import type { ReactElement } from 'react'

export function SimulationMark(): ReactElement {
  return (
    <span
      title="Le cœur Rust est absent : le faux cœur de développement alimente l'interface."
      className="pointer-events-none rounded bg-warn/12 px-1.5 py-px text-[9.5px] tracking-[0.08em]
        text-warn uppercase"
    >
      simulation
    </span>
  )
}
