// Responsabilite : le menu contextuel de la page — une liste d'entrees, posee au-dessus du
// contenu par le coeur. La page n'a que la taille du menu : elle ne peut rien afficher d'autre.

import { motion } from 'framer-motion'
import { useEffect, type ReactElement } from 'react'
import type { ContextTarget, MenuEntry, MenuItemKind, UiRequest } from '../shared/contract'

export interface MenuPanelProps {
  target: ContextTarget
  send: (request: UiRequest) => void
}

function Separator(): ReactElement {
  return <div className="my-1 border-t border-hairline" />
}

function Item({ entry, onRun }: { entry: MenuEntry; onRun: (kind: MenuItemKind) => void }): ReactElement {
  const tone = entry.enabled ? 'text-ink hover:shadow-field' : 'text-ink-faint/60 cursor-default'
  return (
    <button
      type="button"
      role="menuitem"
      disabled={!entry.enabled}
      onClick={() => onRun(entry.kind)}
      className={`flex h-7 w-full items-center rounded-[6px] px-2.5 text-left text-[12px]
        transition-colors duration-100 ${tone}`}
    >
      {entry.label}
    </button>
  )
}

/** Ferme le menu a Echap ou des que la page perd le focus : un menu orphelin reste sinon. */
function useDismiss(close: () => void): void {
  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') {
        event.preventDefault()
        close()
      }
    }
    window.addEventListener('keydown', onKey)
    window.addEventListener('blur', close)
    return () => {
      window.removeEventListener('keydown', onKey)
      window.removeEventListener('blur', close)
    }
  }, [close])
}

export function MenuPanel({ target, send }: MenuPanelProps): ReactElement {
  const close = (): void => send({ kind: 'closeContextMenu' })
  const run = (action: MenuItemKind): void => send({ kind: 'runContextMenu', action })
  useDismiss(close)
  return (
    <motion.div
      role="menu"
      initial={{ opacity: 0, scale: 0.97, y: -3 }}
      animate={{ opacity: 1, scale: 1, y: 0 }}
      transition={{ duration: 0.12, ease: [0.22, 1, 0.36, 1] }}
      className="h-full w-full origin-top-left overflow-hidden border border-hairline bg-card p-1"
    >
      {target.entries.map((entry, index) =>
        entry.separator ? (
          // Un separateur n'a pas d'identite propre : sa position dans la liste suffit.
          <Separator key={`trait-${index}`} />
        ) : (
          <Item key={entry.kind} entry={entry} onRun={run} />
        ),
      )}
    </motion.div>
  )
}
