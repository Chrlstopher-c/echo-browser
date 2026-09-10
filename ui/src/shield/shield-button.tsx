// Responsabilite : bouton bouclier de la rangee d'outils — etat lisible d'un coup d'oeil, compteur.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { ShieldView } from '../shared/contract'
import { IconButton, type ButtonTone } from '../shared/design/icon-button'
import { IconShield, IconShieldCheck } from '../shared/design/icons'
import { formatCount, SHIELD_LABEL, shieldModeOf, type ShieldMode } from './shield-reading'

export interface ShieldButtonProps {
  shield: ShieldView
  open: boolean
  compact: boolean
  onClick: () => void
}

const TONE: Record<ShieldMode, ButtonTone> = { active: 'guard', siteOff: 'neutral', globalOff: 'warn' }

export function ShieldButton({ shield, open, compact, onClick }: ShieldButtonProps): ReactElement {
  const mode = shieldModeOf(shield)
  const showCount = !compact && mode === 'active' && shield.blockedHere > 0
  const count = showCount ? (
    <motion.span
      key={shield.blockedHere}
      initial={{ opacity: 0.4 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.15 }}
      className="numerique text-[11px] font-medium"
    >
      {formatCount(shield.blockedHere)}
    </motion.span>
  ) : undefined
  return (
    <IconButton label={SHIELD_LABEL[mode]} onClick={onClick} tone={TONE[mode]} active={open} trailing={count}>
      {mode === 'active' ? <IconShieldCheck size={15} /> : <IconShield size={15} />}
    </IconButton>
  )
}
