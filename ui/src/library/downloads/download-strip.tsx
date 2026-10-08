// Responsabilite : telechargements en cours, au pied de la barre — nom, progression, annuler. Visible tant qu'un
// fichier arrive ; les termines passent par la bulle « Téléchargé » puis la Bibliotheque.

import type { ReactElement } from 'react'
import { IconButton } from '../../shared/design/icon-button'
import { IconClose } from '../../shared/design/icons'
import { formatPercent } from '../../shared/format'
import { progressOf, stateLabel } from './download-reading'
import type { DownloadsController } from './use-downloads'

export function DownloadStrip({ downloads }: { downloads: DownloadsController }): ReactElement | null {
  const running = downloads.items.filter((item) => item.state === 'running')
  if (running.length === 0) return null
  return (
    <div aria-label="Téléchargements en cours" className="flex flex-col gap-1">
      {running.slice(0, 3).map((item) => {
        const progress = progressOf(item)
        return (
          <div key={item.id} className="flex flex-col gap-1 rounded-row bg-card px-2.5 py-1.5 shadow-card">
            <div className="flex items-center gap-2">
              <p className="min-w-0 flex-1 truncate text-[11.5px] text-ink">{item.fileName}</p>
              <span className="numerique shrink-0 text-[10.5px] text-ink-muted">
                {progress === null ? stateLabel(item) : formatPercent(progress)}
              </span>
              <IconButton label="Annuler le téléchargement" onClick={() => downloads.cancel(item.id)}>
                <IconClose size={11} />
              </IconButton>
            </div>
            <div className="h-1 overflow-hidden rounded-full bg-field" role="progressbar"
              aria-valuenow={progress === null ? undefined : Math.round(progress * 100)}>
              <div className="h-full rounded-full bg-guard transition-[width] duration-300"
                style={{ width: `${Math.round((progress ?? 0.3) * 100)}%` }} />
            </div>
          </div>
        )
      })}
    </div>
  )
}
