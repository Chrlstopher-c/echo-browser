// Responsabilite : la barre laterale — toujours la colonne entiere. Repliee, le coeur la retire de la
// disposition et ne la montre que par-dessus la page, quand la souris longe le bord gauche.

import type { ReactElement } from 'react'
import { SidebarColumn } from './sidebar-column'
import type { SidebarModel } from './use-sidebar'
import { useRevealOnHover } from './use-reveal-on-hover'
import { SIDEBAR_WIDTH } from './sidebar-geometry'

export function Sidebar({ model }: { model: SidebarModel }): ReactElement {
  const hover = useRevealOnHover(model.core.send, model.width.collapsed)
  return (
    <aside
      onPointerEnter={hover.enter}
      onPointerLeave={hover.leave}
      // Un clic dans la barre referme le menu contextuel d'une page, s'il est ouvert.
      onPointerDownCapture={() => model.core.send({ kind: 'closeContextMenu' })}
      style={{ width: SIDEBAR_WIDTH }}
      className="fond-espace relative h-full shrink-0 overflow-hidden"
    >
      <SidebarColumn model={model} />
    </aside>
  )
}
