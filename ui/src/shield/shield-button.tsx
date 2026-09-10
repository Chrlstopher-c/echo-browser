// Responsabilite : bouton bouclier de la barre de navigation — etat lisible d'un coup d'oeil.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { ShieldView } from '../shared/contract'
import { IconShield, IconShieldCheck } from '../shared/design/icons'
import { formatCount, SHIELD_LABEL, shieldModeOf, type ShieldMode } from './shield-reading'

export interface ShieldButtonProps {
  shield: ShieldView
  open: boolean
  onClick: () => void
}

const SKIN: Record<ShieldMode, string> = {
  active: 'text-guard bg-guard/12 hover:bg-guard/18',
  siteOff: 'text-ink-faint hover:bg-hover',
  globalOff: 'text-warn bg-warn/10 hover:bg-warn/16',
}

export function ShieldButton({ shield, open, onClick }: ShieldButtonProps): ReactElement {
  const mode = shieldModeOf(shield)
  const showCount = mode === 'active' && shield.blockedHere > 0
  return (
    <button
      type="button"
      onClick={onClick}
      aria-label={SHIELD_LABEL[mode]}
      aria-expanded={open}
      title={SHIELD_LABEL[mode]}
      className={`flex h-7 shrink-0 items-center gap-1.5 rounded-md px-1.5 transition-colors duration-150
        ${SKIN[mode]} ${open ? 'ring-1 ring-edge' : ''}`}
    >
      {mode === 'active' ? <IconShieldCheck size={15} /> : <IconShield size={15} />}
      {showCount && (
        <motion.span
          key={shield.blockedHere}
          initial={{ opacity: 0.4 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.15 }}
          className="numerique pr-0.5 text-[11px] font-medium"
        >
          {formatCount(shield.blockedHere)}
        </motion.span>
      )}
    </button>
  )
}
