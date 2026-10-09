// Responsabilite : la section affichee, lue dans l'ancre de l'adresse (#reglages, #bibliotheque, #admin).

import { useEffect, useState } from 'react'

export type PageId = 'reglages' | 'bibliotheque' | 'extensions' | 'bienvenue' | 'admin' | 'aide' | 'effacer'

function read(): PageId {
  const hash = window.location.hash.slice(1)
  const known: PageId[] = ['bibliotheque', 'extensions', 'bienvenue', 'admin', 'aide', 'effacer']
  return known.find((page) => page === hash) ?? 'reglages'
}

export function usePageRoute(): [PageId, (next: PageId) => void] {
  const [page, setPage] = useState<PageId>(read)
  useEffect(() => {
    const onHash = (): void => setPage(read())
    window.addEventListener('hashchange', onHash)
    return () => window.removeEventListener('hashchange', onHash)
  }, [])
  const go = (next: PageId): void => {
    window.location.hash = next
  }
  return [page, go]
}
