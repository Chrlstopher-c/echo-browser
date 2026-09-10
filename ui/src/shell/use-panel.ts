// Responsabilite : ouverture des panneaux et hauteur reclamee au coeur. Un panneau ouvert a la fois.

import { useCallback, useEffect, useState } from 'react'
import type { UiRequest } from '../shared/contract'
import { chromeHeightFor, type PanelId } from './panel'

export interface PanelController {
  open: PanelId | null
  toggle: (panel: PanelId) => void
  show: (panel: PanelId) => void
  close: () => void
}

export function usePanel(send: (request: UiRequest) => void): PanelController {
  const [open, setOpen] = useState<PanelId | null>(null)

  useEffect(() => {
    send({ kind: 'setChromeHeight', pixels: chromeHeightFor(open) })
  }, [open, send])

  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') setOpen(null)
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])

  const toggle = useCallback((panel: PanelId): void => {
    setOpen((current) => (current === panel ? null : panel))
  }, [])

  const show = useCallback((panel: PanelId): void => setOpen(panel), [])
  const close = useCallback((): void => setOpen(null), [])

  return { open, toggle, show, close }
}
