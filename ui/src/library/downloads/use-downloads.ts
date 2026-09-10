// Responsabilite : commandes des telechargements — ouvrir, reveler, annuler, oublier.

import { useCallback, useMemo } from 'react'
import type { DownloadId, DownloadView, UiRequest } from '../../shared/contract'
import { summarize, type DownloadsSummary } from './download-reading'

export interface DownloadsController {
  items: DownloadView[]
  summary: DownloadsSummary
  open: (id: DownloadId) => void
  reveal: (id: DownloadId) => void
  cancel: (id: DownloadId) => void
  forget: (id: DownloadId) => void
  /** Oublie toutes les entrees terminees, annulees ou en echec. */
  clearFinished: () => void
}

type Send = (request: UiRequest) => void

export function useDownloads(send: Send, items: DownloadView[]): DownloadsController {
  const summary = useMemo(() => summarize(items), [items])
  const open = useCallback((id: DownloadId): void => send({ kind: 'openDownload', id }), [send])
  const reveal = useCallback((id: DownloadId): void => send({ kind: 'revealDownload', id }), [send])
  const cancel = useCallback((id: DownloadId): void => send({ kind: 'cancelDownload', id }), [send])
  const forget = useCallback((id: DownloadId): void => send({ kind: 'forgetDownload', id }), [send])
  const clearFinished = useCallback((): void => {
    for (const item of items) {
      if (item.state !== 'running' && item.state !== 'paused') send({ kind: 'forgetDownload', id: item.id })
    }
  }, [send, items])
  return { items, summary, open, reveal, cancel, forget, clearFinished }
}
