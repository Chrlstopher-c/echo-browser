// Responsabilite : commandes du panneau des extensions — installation par le catalogue, bascule,
// retrait confirme, gestionnaire de Chromium, relance.

import { useCallback, useMemo, useState } from 'react'
import type { ExtensionView, UiRequest } from '../shared/contract'
import { pendingCount, STORE_URL } from './extension-model'
import { readExtensionSource } from './extension-source'

export interface InstallField {
  value: string
  error: string | null
  change: (next: string) => void
  submit: () => void
}

export interface Removal {
  /** Extension dont le retrait attend confirmation, ou null. */
  confirming: string | null
  askRemove: (id: string) => void
  cancelRemove: () => void
  confirmRemove: (id: string) => void
}

export interface ExtensionsController extends Removal {
  extensions: ExtensionView[]
  restartPending: boolean
  pending: number
  install: InstallField
  setEnabled: (id: string, enabled: boolean) => void
  openStore: () => void
  openManager: () => void
  restart: () => void
}

type Send = (request: UiRequest) => void

const ALREADY_HERE = 'Cette extension est déjà installée.'

function useInstallField(send: Send, installed: string[]): InstallField {
  const [value, setValue] = useState('')
  const [error, setError] = useState<string | null>(null)

  const change = useCallback((next: string): void => {
    setValue(next)
    setError(null)
  }, [])

  const submit = useCallback((): void => {
    const check = readExtensionSource(value)
    if (!check.ok) {
      setError(check.reason)
      return
    }
    if (installed.includes(check.id)) {
      setError(ALREADY_HERE)
      return
    }
    send({ kind: 'installExtension', source: check.source })
    setValue('')
  }, [value, installed, send])

  return { value, error, change, submit }
}

function useRemoval(send: Send): Removal {
  const [confirming, setConfirming] = useState<string | null>(null)
  const askRemove = useCallback((id: string): void => setConfirming(id), [])
  const cancelRemove = useCallback((): void => setConfirming(null), [])
  const confirmRemove = useCallback(
    (id: string): void => {
      setConfirming(null)
      send({ kind: 'removeExtension', id })
    },
    [send],
  )
  return { confirming, askRemove, cancelRemove, confirmRemove }
}

export function useExtensions(send: Send, extensions: ExtensionView[], restartPending: boolean): ExtensionsController {
  const installed = useMemo(() => extensions.map((item) => item.id), [extensions])
  const install = useInstallField(send, installed)
  const removal = useRemoval(send)
  const pending = useMemo(() => pendingCount(extensions), [extensions])

  const setEnabled = useCallback(
    (id: string, enabled: boolean): void => send({ kind: 'setExtensionEnabled', id, enabled }),
    [send],
  )
  const openStore = useCallback((): void => send({ kind: 'installExtension', source: STORE_URL }), [send])
  const openManager = useCallback((): void => send({ kind: 'openExtensionManager' }), [send])
  const restart = useCallback((): void => send({ kind: 'restartBrowser' }), [send])

  return { ...removal, extensions, restartPending, pending, install, setEnabled, openStore, openManager, restart }
}
