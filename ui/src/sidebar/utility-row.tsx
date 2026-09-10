// Responsabilite : rangee d'outils au bas de la barre — bouclier, extensions, telechargements, reglages.

import type { ReactElement } from 'react'
import type { ShieldView } from '../shared/contract'
import { IconButton } from '../shared/design/icon-button'
import { IconDownload, IconPuzzle, IconSettings } from '../shared/design/icons'
import { ShieldButton } from '../shield/shield-button'
import type { SheetId } from './sheet'

export interface UtilityRowProps {
  shield: ShieldView
  open: SheetId | null
  compact: boolean
  /** Un changement d'extension attend la relance : le bouton porte une pastille. */
  restartPending: boolean
  onToggle: (sheet: SheetId) => void
}

function PendingDot(): ReactElement {
  return <span aria-hidden className="absolute top-1 right-1 size-1.5 rounded-full bg-warn" />
}

export function UtilityRow(props: UtilityRowProps): ReactElement {
  const { shield, open, compact, restartPending, onToggle } = props
  return (
    <div className={`flex items-center gap-0.5 ${compact ? 'flex-col' : ''}`}>
      <ShieldButton shield={shield} open={open === 'shield'} compact={compact} onClick={() => onToggle('shield')} />
      {!compact && <span className="flex-1" />}
      <span className="relative flex">
        <IconButton label="Extensions" onClick={() => onToggle('extensions')} active={open === 'extensions'}>
          <IconPuzzle size={15} />
        </IconButton>
        {restartPending && <PendingDot />}
      </span>
      <IconButton label="Téléchargements" onClick={() => onToggle('downloads')} active={open === 'downloads'}>
        <IconDownload size={15} />
      </IconButton>
      <IconButton label="Réglages" onClick={() => onToggle('settings')} active={open === 'settings'}>
        <IconSettings size={15} />
      </IconButton>
    </div>
  )
}
