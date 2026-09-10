// Responsabilite : anneau de progression — un arc qui suit une valeur reelle de 0 a 1, sans a-coup.
// Sert au chargement d'un onglet, au bouton arreter, et au badge des telechargements.

import { motion } from 'framer-motion'
import type { ReactElement } from 'react'
import { TRACK } from './motion'

export interface ProgressRingProps {
  /** De 0 a 1. */
  progress: number
  size: number
  stroke?: number
  className?: string
}

export function ProgressRing({ progress, size, stroke = 1.5, className = '' }: ProgressRingProps): ReactElement {
  const radius = (size - stroke) / 2
  const circumference = 2 * Math.PI * radius
  const ratio = Math.min(Math.max(progress, 0), 1)
  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden="true"
      className={`shrink-0 -rotate-90 ${className}`}>
      <circle cx={size / 2} cy={size / 2} r={radius} fill="none" stroke="currentColor" strokeWidth={stroke}
        className="opacity-20" />
      <motion.circle
        cx={size / 2}
        cy={size / 2}
        r={radius}
        fill="none"
        stroke="currentColor"
        strokeWidth={stroke}
        strokeLinecap="round"
        strokeDasharray={circumference}
        initial={{ strokeDashoffset: circumference }}
        animate={{ strokeDashoffset: circumference * (1 - ratio) }}
        transition={TRACK}
      />
    </svg>
  )
}
