// Responsabilite : cadre commun des panneaux — entete, action facultative, corps defilant.

import type { ReactElement, ReactNode } from 'react'

export interface PanelShellProps {
  title: string
  children: ReactNode
  action?: ReactNode
  footer?: ReactNode
}

export function PanelShell({ title, children, action, footer }: PanelShellProps): ReactElement {
  return (
    <section className="flex h-full flex-col">
      <header className="flex h-8 shrink-0 items-center justify-between border-b border-hairline px-3">
        <h2 className="text-[11px] font-medium tracking-[0.07em] text-ink-muted uppercase">{title}</h2>
        {action}
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto">{children}</div>
      {footer !== undefined && (
        <footer className="shrink-0 border-t border-hairline px-3 py-2 text-[11px] text-ink-faint">
          {footer}
        </footer>
      )}
    </section>
  )
}

export interface PanelRowProps {
  children: ReactNode
  onClick?: () => void
}

/** Ligne cliquable standard d'un panneau de liste. */
export function PanelRow({ children, onClick }: PanelRowProps): ReactElement {
  const interactive = onClick !== undefined
  return (
    <div
      role={interactive ? 'button' : undefined}
      tabIndex={interactive ? 0 : undefined}
      onClick={onClick}
      className={`flex items-center gap-2.5 px-3 py-[7px] text-[12.5px]
        ${interactive ? 'cursor-default hover:bg-hover' : ''}`}
    >
      {children}
    </div>
  )
}
