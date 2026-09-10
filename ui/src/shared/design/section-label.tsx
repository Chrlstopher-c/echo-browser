// Responsabilite : intitule de section d'une feuille — petites capitales, appendice optionnel a droite.

import type { ReactElement, ReactNode } from 'react'

export function SectionLabel({ children, aside }: { children: ReactNode; aside?: ReactNode }): ReactElement {
  return (
    <div className="flex h-7 items-center justify-between gap-2 px-2">
      <p className="intitule truncate">{children}</p>
      {aside !== undefined && <span className="shrink-0">{aside}</span>}
    </div>
  )
}
