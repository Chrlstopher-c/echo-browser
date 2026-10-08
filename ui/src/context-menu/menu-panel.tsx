// Responsabilite : le menu contextuel de la page — une liste d'entrees, posee au-dessus du
// contenu par le coeur. La page n'a que la taille du menu : elle ne peut rien afficher d'autre.

import { motion } from 'framer-motion'
import { useEffect, type ReactElement } from 'react'
import type { ContextTarget, MenuEntry, MenuItemKind, UiRequest } from '../shared/contract'
import {
  IconBack, IconClipboard, IconCode, IconCopy, IconDownload, IconFile, IconForward, IconImage, IconOpen, IconPlus,
  IconPrinter, IconReload, IconScissors, IconSearch, IconShield, IconStar, IconWrench,
  type IconComponent,
} from '../shared/design/icons'
import { IconEye, IconEyeOff } from '../shared/design/icons-page'

export interface MenuPanelProps {
  target: ContextTarget
  send: (request: UiRequest) => void
}

function Separator(): ReactElement {
  return <div className="my-1 border-t border-hairline" />
}

/** Meme langage que le menu des onglets : une icone par action, alignee a gauche. */
const ICONS: Partial<Record<MenuItemKind, IconComponent>> = {
  openLinkInTab: IconOpen, openLinkInBackground: IconPlus, copyLink: IconCopy, saveLink: IconDownload,
  openImage: IconImage, copyImageLink: IconCopy, saveImage: IconDownload, copyImage: IconImage,
  openMedia: IconOpen, copyMediaLink: IconCopy, saveMedia: IconDownload,
  copy: IconCopy, cut: IconScissors, paste: IconClipboard, pastePlain: IconClipboard, selectAll: IconFile,
  searchSelection: IconSearch, openSelection: IconOpen,
  back: IconBack, forward: IconForward, reload: IconReload, copyPageLink: IconCopy, bookmark: IconStar,
  savePage: IconDownload, print: IconPrinter, toggleShield: IconShield, viewSource: IconCode, inspect: IconWrench,
  hideElement: IconEyeOff, unhideElements: IconEye,
}

function Item({ entry, onRun }: { entry: MenuEntry; onRun: (kind: MenuItemKind) => void }): ReactElement {
  const tone = entry.enabled ? 'text-ink hover:bg-hover hover:shadow-field' : 'text-ink-faint/60 cursor-default'
  const Icon = ICONS[entry.kind]
  return (
    <button
      type="button"
      role="menuitem"
      disabled={!entry.enabled}
      onClick={() => onRun(entry.kind)}
      className={`flex h-7 w-full items-center gap-2.5 rounded-[7px] px-2 text-left text-[12px]
        transition-[background-color,box-shadow] duration-100 ${tone}`}
    >
      <span className={entry.enabled ? 'text-ink-muted' : ''}>{Icon !== undefined && <Icon size={13} />}</span>
      <span className="truncate">{entry.label}</span>
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
      className="h-full w-full origin-top-left overflow-hidden border border-hairline bg-card p-1
        shadow-[inset_1px_1px_0_var(--color-hi),inset_-1px_-1px_0_var(--color-lo)]"
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
