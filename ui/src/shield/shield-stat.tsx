// Responsabilite : chiffre de blocage — grand nombre monospace et sa legende.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { formatCount } from './shield-reading'

export interface ShieldStatProps {
  value: number
  caption: string
  strong?: boolean
}

export function ShieldStat({ value, caption, strong = false }: ShieldStatProps): ReactElement {
  return (
    <div className="flex flex-col gap-0.5 rounded-panel bg-inset/70 px-3 py-2.5">
      <motion.span
        key={value}
        initial={{ opacity: 0.55 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.18 }}
        className={`numerique text-[19px] leading-none ${strong ? 'text-guard' : 'text-ink'}`}
      >
        {formatCount(value)}
      </motion.span>
      <span className="text-[11px] text-ink-faint">{caption}</span>
    </div>
  )
}
