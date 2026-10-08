// Responsabilite : icones des outils de page — activite reseau, masquer et reafficher un element, lecture. Meme
// trace que icons.tsx (Glyph partage).

import type { ReactElement } from 'react'
import { Glyph, type IconProps } from './icons'

/** Activite reseau : une trace de pouls. */
export function IconActivity(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M1.5 8h2.7l1.8-4.6 3.3 9.2 1.8-4.6h3.4" />
    </Glyph>
  )
}

/** Masquer : un oeil barre. */
export function IconEyeOff(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M2 8s2.2-4 6-4 6 4 6 4-2.2 4-6 4-6-4-6-4Z" />
      <circle cx="8" cy="8" r="1.6" />
      <path d="M2.5 13.5 13.5 2.5" />
    </Glyph>
  )
}

/** Reafficher : un oeil ouvert. */
export function IconEye(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M2 8s2.2-4 6-4 6 4 6 4-2.2 4-6 4-6-4-6-4Z" />
      <circle cx="8" cy="8" r="1.6" />
    </Glyph>
  )
}

/** Mode lecture : un livre ouvert. */
export function IconReader(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M8 4.2C6.6 3.2 4.6 2.8 2 3v9.3c2.6-.2 4.6.2 6 1.2 1.4-1 3.4-1.4 6-1.2V3c-2.6-.2-4.6.2-6 1.2Z" />
      <path d="M8 4.2v9.3" />
    </Glyph>
  )
}
