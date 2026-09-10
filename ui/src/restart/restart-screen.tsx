// Responsabilite : ecran d'attente plein cadre pendant la relance du navigateur. Il vit quelques
// secondes, puis le processus est remplace : aucune sortie a gerer, il disparait avec la page.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { RESTART_HINT } from '../extensions/extension-model'
import { IconReload } from '../shared/design/icons'
import { CHROME_EASE } from '../shared/design/motion'

/** Va-et-vient de la barre indeterminee, en secondes. */
const SWEEP_SECONDS = 1.1

function Sweep(): ReactElement {
  return (
    <div className="relative h-[3px] w-52 overflow-hidden rounded-full bg-hairline" role="progressbar">
      <motion.span
        initial={{ x: '-100%' }}
        animate={{ x: '260%' }}
        transition={{ duration: SWEEP_SECONDS, ease: 'easeInOut', repeat: Infinity }}
        className="absolute inset-y-0 w-1/3 rounded-full bg-guard"
      />
    </div>
  )
}

export function RestartScreen({ reason }: { reason: string }): ReactElement {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.2, ease: CHROME_EASE }}
      role="status"
      aria-live="polite"
      className="fond-espace fixed inset-0 z-50 flex flex-col items-center justify-center gap-5 px-10 text-center"
    >
      <motion.span
        animate={{ rotate: 360 }}
        transition={{ duration: 2.4, ease: 'linear', repeat: Infinity }}
        className="grid size-11 place-items-center rounded-full bg-card text-guard shadow-card"
      >
        <IconReload size={20} />
      </motion.span>
      <div className="flex flex-col gap-1.5">
        <p className="text-[15px] font-medium text-ink">Relance du navigateur</p>
        <p className="max-w-[46ch] text-[12.5px] leading-relaxed text-ink-muted">{reason}</p>
      </div>
      <Sweep />
      <p className="text-[11.5px] text-ink-faint">{RESTART_HINT}</p>
    </motion.div>
  )
}
