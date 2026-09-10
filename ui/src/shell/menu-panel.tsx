// Responsabilite : menu principal — acces aux panneaux bibliotheque, extensions et outils.

import type { ReactElement, ReactNode } from 'react'
import { IconChevron, IconClock, IconDownload, IconPuzzle, IconStar } from '../shared/design/icons'
import { PanelShell } from '../shared/design/panel-shell'
import type { PanelId } from './panel'

export interface MenuPanelProps {
  onOpen: (panel: PanelId) => void
  onDevTools: () => void
}

interface MenuEntry {
  panel: PanelId
  label: string
  icon: ReactNode
}

const ENTRIES: MenuEntry[] = [
  { panel: 'bookmarks', label: 'Favoris', icon: <IconStar size={15} /> },
  { panel: 'history', label: 'Historique', icon: <IconClock size={15} /> },
  { panel: 'downloads', label: 'Téléchargements', icon: <IconDownload size={15} /> },
  { panel: 'extensions', label: 'Extensions', icon: <IconPuzzle size={15} /> },
]

export function MenuPanel({ onOpen, onDevTools }: MenuPanelProps): ReactElement {
  return (
    <PanelShell title="Menu">
      <div className="py-1">
        {ENTRIES.map((entry) => (
          <button
            key={entry.panel}
            type="button"
            onClick={() => onOpen(entry.panel)}
            className="flex w-full items-center gap-2.5 px-3.5 py-[7px] text-left text-[12.5px]
              text-ink transition-colors duration-100 hover:bg-hover"
          >
            <span className="text-ink-muted">{entry.icon}</span>
            <span className="flex-1">{entry.label}</span>
            <IconChevron size={13} className="text-ink-faint" />
          </button>
        ))}
        <div className="my-1 border-t border-hairline" />
        <button
          type="button"
          onClick={onDevTools}
          className="flex w-full items-center px-3.5 py-[7px] text-left text-[12.5px] text-ink-muted
            transition-colors duration-100 hover:bg-hover hover:text-ink"
        >
          Outils de développement
        </button>
      </div>
    </PanelShell>
  )
}
