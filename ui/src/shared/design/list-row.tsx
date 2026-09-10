// Responsabilite : ligne standard d'une liste de feuille — icone, contenu, actions au survol.

import type { KeyboardEvent, ReactElement, ReactNode } from 'react'

export interface ListRowProps {
  children: ReactNode
  onClick?: () => void
  /** Titre complet, montre en infobulle. */
  title?: string
  /** Hauteur : 8 (32px) pour une ligne simple, 11 (44px) pour deux lignes. */
  height?: 8 | 11
}

export function ListRow({ children, onClick, title, height = 8 }: ListRowProps): ReactElement {
  const interactive = onClick !== undefined
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>): void => {
    if (event.key === 'Enter' && onClick !== undefined) onClick()
  }
  return (
    <div
      role={interactive ? 'button' : undefined}
      tabIndex={interactive ? 0 : undefined}
      title={title}
      onClick={onClick}
      onKeyDown={interactive ? onKeyDown : undefined}
      className={`group flex items-center gap-2.5 rounded-row px-2 text-[12.5px]
        ${height === 8 ? 'h-8' : 'h-11'}
        ${interactive ? 'transition-colors duration-100 hover:bg-hover' : ''}`}
    >
      {children}
    </div>
  )
}

/** Action revelee au survol de la ligne : invisible au repos, pour garder la liste calme. */
export function RowAction(props: { label: string; onClick: () => void; children: ReactNode; danger?: boolean }):
  ReactElement {
  const { label, onClick, children, danger = false } = props
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={(event) => {
        event.stopPropagation()
        onClick()
      }}
      className={`grid size-6 shrink-0 place-items-center rounded-md text-ink-faint opacity-0
        transition-[opacity,color,background-color] duration-100 group-hover:opacity-100 hover:bg-ink/10
        focus-visible:opacity-100 ${danger ? 'hover:text-danger' : 'hover:text-ink'}`}
    >
      {children}
    </button>
  )
}
