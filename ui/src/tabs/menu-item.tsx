// Responsabilite : une ligne de menu contextuel de la barre, et son separateur.

import type { ReactElement, ReactNode } from 'react'
import { IconCheck } from '../shared/design/icons'

export function MenuItem({ icon, label, onClick, danger = false, checked }: {
  icon: ReactNode; label: string; onClick: () => void; danger?: boolean
  /** Option a cocher : la coche s'affiche a droite quand elle est active. */
  checked?: boolean
}): ReactElement {
  return (
    <button
      type="button"
      role={checked === undefined ? 'menuitem' : 'menuitemcheckbox'}
      aria-checked={checked}
      onClick={onClick}
      className={`flex h-7 w-full items-center gap-2.5 rounded-[6px] px-2 text-left text-[12px]
        transition-colors duration-100 hover:bg-hover
        ${danger ? 'text-ink-muted hover:text-danger' : 'text-ink'}`}
    >
      <span className="text-ink-muted">{icon}</span>
      <span className="flex-1 truncate">{label}</span>
      {checked === true && <IconCheck size={12} className="text-ink-muted" />}
    </button>
  )
}

export function MenuSeparator(): ReactElement {
  return <div className="my-1 border-t border-hairline" />
}
