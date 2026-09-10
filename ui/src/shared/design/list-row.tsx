// Responsabilite : ligne standard d'une liste de feuille — icone, contenu, appendice.

import type { ReactElement, ReactNode } from 'react'

export interface ListRowProps {
  children: ReactNode
  onClick?: () => void
}

export function ListRow({ children, onClick }: ListRowProps): ReactElement {
  const interactive = onClick !== undefined
  return (
    <div
      role={interactive ? 'button' : undefined}
      tabIndex={interactive ? 0 : undefined}
      onClick={onClick}
      className={`flex h-8 items-center gap-2.5 rounded-row px-2 text-[12.5px]
        ${interactive ? 'transition-colors duration-100 hover:bg-hover' : ''}`}
    >
      {children}
    </div>
  )
}
