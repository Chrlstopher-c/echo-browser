// Responsabilite : commandes du panneau des extensions — installation, bascule, retrait, relance.

import { useCallback, useMemo, useState } from 'react'
import type { ExtensionView, UiRequest } from '../shared/contract'
import { inventorySignature, pendingCount } from './extension-model'
import { useInstall, type InstallController } from './use-install'

export interface ExtensionsController {
  extensions: ExtensionView[]
  restartPending: boolean
  pending: number
  install: InstallController
  /** Extension dont le retrait attend confirmation, ou null. */
  confirming: string | null
  setEnabled: (id: string, enabled: boolean) => void
  askRemove: (id: string) => void
  cancelRemove: () => void
  confirmRemove: (id: string) => void
  restart: () => void
}

type Send = (request: UiRequest) => void

export function useExtensions(send: Send, extensions: ExtensionView[], restartPending: boolean): ExtensionsController {
  const signature = useMemo(() => inventorySignature(extensions), [extensions])
  const installed = useMemo(() => extensions.map((item) => item.id), [extensions])
  const install = useInstall({ send, signature, installed })
  const [confirming, setConfirming] = useState<string | null>(null)

  const setEnabled = useCallback(
    (id: string, enabled: boolean): void => send({ kind: 'setExtensionEnabled', id, enabled }),
    [send],
  )
  const askRemove = useCallback((id: string): void => setConfirming(id), [])
  const cancelRemove = useCallback((): void => setConfirming(null), [])
  const confirmRemove = useCallback(
    (id: string): void => {
      setConfirming(null)
      send({ kind: 'removeExtension', id })
    },
    [send],
  )
  const restart = useCallback((): void => send({ kind: 'restartBrowser' }), [send])

  return {
    extensions,
    restartPending,
    pending: useMemo(() => pendingCount(extensions), [extensions]),
    install,
    confirming,
    setEnabled,
    askRemove,
    cancelRemove,
    confirmRemove,
    restart,
  }
}
