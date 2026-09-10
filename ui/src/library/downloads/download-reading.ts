// Responsabilite : lecture d'un telechargement — progression, libelle d'etat, agregat pour le badge.

import type { DownloadView } from '../../shared/contract'
import { formatBytes, formatRelative } from '../../shared/format'

/** Part recue, ou null quand la taille totale est inconnue. */
export function progressOf(item: DownloadView): number | null {
  if (item.total === null || item.total <= 0) return null
  return Math.min(1, item.received / item.total)
}

export function stateLabel(item: DownloadView, now: number = Date.now()): string {
  switch (item.state) {
    case 'running': {
      const total = item.total === null ? '' : ` sur ${formatBytes(item.total)}`
      return `${formatBytes(item.received)}${total}`
    }
    case 'paused':
      return `En pause · ${formatBytes(item.received)} reçus`
    case 'complete':
      return `${formatBytes(item.received)} · ${formatRelative(item.startedAt, now)}`
    case 'cancelled':
      return 'Annulé'
    case 'failed':
      return `Échec · ${formatBytes(item.received)} reçus`
  }
}

export interface DownloadsSummary {
  running: number
  /** Progression moyenne des telechargements en cours, ou null si aucun n'a de taille connue. */
  progress: number | null
}

export function summarize(items: DownloadView[]): DownloadsSummary {
  const running = items.filter((item) => item.state === 'running')
  const known = running.map(progressOf).filter((value): value is number => value !== null)
  const progress = known.length === 0 ? null : known.reduce((sum, value) => sum + value, 0) / known.length
  return { running: running.length, progress }
}
