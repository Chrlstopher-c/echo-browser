// Responsabilite : chiffre de blocage — nombre monospace et sa legende complete.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { formatCount } from '../shared/format'

export interface ShieldStatProps {
  value: number
  caption: string
  strong?: boolean
}

export function ShieldStat({ value, caption, strong = false }: ShieldStatProps): ReactElement {
  return (
    <div className="flex flex-col gap-0.5 rounded-row bg-card px-2.5 py-2 shadow-card">
      <motion.span
        key={value}
        initial={{ opacity: 0.55 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.18 }}
        className={`numerique text-[17px] leading-none ${strong ? 'text-guard' : 'text-ink'}`}
      >
        {formatCount(value)}
      </motion.span>
      <span className="text-[10.5px] leading-tight text-ink-faint">{caption}</span>
    </div>
  )
}
