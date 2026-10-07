// Responsabilite : la section affichee, lue dans l'ancre de l'adresse (#reglages, #bibliotheque).

import { useEffect, useState } from 'react'

export type PageId = 'reglages' | 'bibliotheque'

function read(): PageId {
  return window.location.hash === '#bibliotheque' ? 'bibliotheque' : 'reglages'
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
