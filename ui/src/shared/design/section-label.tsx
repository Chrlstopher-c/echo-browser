// Responsabilite : intitule de section de la barre — petites capitales, discret.

import type { ReactElement, ReactNode } from 'react'

export function SectionLabel({ children }: { children: ReactNode }): ReactElement {
  return (
    <p className="px-2 pt-1 pb-1.5 text-[10px] font-medium tracking-[0.1em] text-ink-faint uppercase">
      {children}
    </p>
  )
}
