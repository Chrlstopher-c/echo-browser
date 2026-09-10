// Responsabilite : etat vide d'un panneau — pas d'illustration, une phrase et un motif discret.

import type { ReactElement, ReactNode } from 'react'

export interface EmptyStateProps {
  icon: ReactNode
  title: string
  hint: string
}

export function EmptyState({ icon, title, hint }: EmptyStateProps): ReactElement {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 px-8 text-center">
      <span className="text-ink-faint">{icon}</span>
      <p className="text-[12.5px] text-ink-muted">{title}</p>
      <p className="max-w-[42ch] text-[11.5px] leading-relaxed text-ink-faint">{hint}</p>
    </div>
  )
}
