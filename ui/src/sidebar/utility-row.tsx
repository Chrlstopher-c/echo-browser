// Responsabilite : rangee d'outils au bas de la barre — bouclier, reseau, bibliotheque, extensions, reglages.

import type { ReactElement } from 'react'
import type { ShieldView } from '../shared/contract'
import { DownloadBadge } from '../library/downloads/download-badge'
import type { DownloadsSummary } from '../library/downloads/download-reading'
import { IconButton } from '../shared/design/icon-button'
import { IconLibrary, IconPuzzle, IconSettings, IconTerminal } from '../shared/design/icons'
import { IconActivity } from '../shared/design/icons-page'
import { IconHelp } from '../shared/design/icons'
import { ShieldButton } from '../shield/shield-button'
import type { SheetId } from './sheet'

export interface UtilityRowProps {
  shield: ShieldView
  open: SheetId | null
  compact: boolean
  /** Un changement d'extension attend la relance : le bouton porte une pastille. */
  restartPending: boolean
  downloads: DownloadsSummary
  onToggle: (sheet: SheetId) => void
  onOpenTerminal: () => void
  /** Reglages et bibliotheque s'ouvrent en page pleine largeur, dans un onglet. */
  onOpenPage: (page: 'reglages' | 'bibliotheque' | 'extensions' | 'aide') => void
}

function PendingDot(): ReactElement {
  return <span aria-hidden className="pointer-events-none absolute top-1 right-1 size-1.5 rounded-full bg-warn" />
}

export function UtilityRow(props: UtilityRowProps): ReactElement {
  const { shield, open, compact, restartPending, downloads, onToggle, onOpenTerminal, onOpenPage } = props
  return (
    <div className={`flex items-center gap-0.5 ${compact ? 'flex-col' : ''}`}>
      <ShieldButton shield={shield} open={open === 'shield'} compact={compact} onClick={() => onToggle('shield')} />
      {!compact && <span className="flex-1" />}
      <IconButton label="Sécurité et réseau" onClick={() => onToggle('network')} active={open === 'network'}>
        <IconActivity size={15} />
      </IconButton>
      <IconButton label="Claude Code" onClick={onOpenTerminal}>
        <IconTerminal size={15} />
      </IconButton>
      <IconButton label="Bibliothèque" onClick={() => onOpenPage('bibliotheque')}>
        <IconLibrary size={15} />
        <DownloadBadge summary={downloads} />
      </IconButton>
      <IconButton label="Extensions" onClick={() => onToggle('extensions')} active={open === 'extensions'}>
        <IconPuzzle size={15} />
        {restartPending && <PendingDot />}
      </IconButton>
      <IconButton label="Réglages" onClick={() => onOpenPage('reglages')}>
        <IconSettings size={15} />
      </IconButton>
      <IconButton label="Aide (F1)" onClick={() => onOpenPage('aide')}>
        <IconHelp size={15} />
      </IconButton>
    </div>
  )
}
