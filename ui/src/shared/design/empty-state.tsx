// Responsabilite : etat vide d'un panneau — pas d'illustration, une phrase et un motif discret.

import type { ReactElement, ReactNode } from 'react'

export interface EmptyStateProps {
  icon: ReactNode
  title: string
  hint: string
  action?: ReactNode
}

export function EmptyState({ icon, title, hint, action }: EmptyStateProps): ReactElement {
  return (
    <div className="flex h-full min-h-40 flex-col items-center justify-center gap-2 px-6 text-center">
      <span className="grid size-9 place-items-center rounded-full bg-ink/5 text-ink-faint">{icon}</span>
      <p className="text-[12.5px] text-ink-muted">{title}</p>
      <p className="max-w-[40ch] text-[11.5px] leading-relaxed text-ink-faint">{hint}</p>
      {action !== undefined && <div className="pt-1">{action}</div>}
    </div>
  )
}
