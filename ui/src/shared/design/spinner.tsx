// Responsabilite : indicateur de chargement ponctuel, a la place d'un favicon.

import type { ReactElement } from 'react'

export function Spinner({ size }: { size: number }): ReactElement {
  return (
    <span
      style={{ width: size, height: size }}
      className="shrink-0 rounded-full border-[1.5px] border-hairline border-t-ink-muted motion-safe:animate-spin"
    />
  )
}
