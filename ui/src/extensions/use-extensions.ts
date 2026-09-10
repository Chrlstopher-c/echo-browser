// Responsabilite : source des extensions et bascule locale. Branchement au coeur a venir.

import { useCallback, useState } from 'react'
import { NO_EXTENSIONS, type Extension } from './extension-model'

export interface ExtensionsController {
  extensions: Extension[]
  toggle: (id: string, enabled: boolean) => void
}

export function useExtensions(): ExtensionsController {
  const [extensions, setExtensions] = useState<Extension[]>(NO_EXTENSIONS)

  const toggle = useCallback((id: string, enabled: boolean): void => {
    setExtensions((current) => current.map((item) => (item.id === id ? { ...item, enabled } : item)))
  }, [])

  return { extensions, toggle }
}
