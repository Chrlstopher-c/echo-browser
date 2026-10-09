// Responsabilite : reprendre favoris et historique d'un autre navigateur — liste des navigateurs trouves sur la
// machine, un bouton chacun. Utilise par l'accueil et par les Reglages.

import { useEffect, useState, type ReactElement } from 'react'
import type { ImportSourceView, UiRequest } from '../shared/contract'
import { PushButton } from '../shared/design/push-button'

export interface ImportSectionProps {
  sources: ImportSourceView[] | null
  send: (request: UiRequest) => void
}

export function ImportSection({ sources, send }: ImportSectionProps): ReactElement {
  const [done, setDone] = useState<string[]>([])
  useEffect(() => send({ kind: 'importSources' }), [send])
  if (sources === null) return <p className="text-[12px] text-ink-faint">Recherche des navigateurs installés…</p>
  if (sources.length === 0) {
    return <p className="text-[12px] text-ink-muted">Aucun autre navigateur trouvé sur cet ordinateur.</p>
  }
  return (
    <div className="flex flex-col gap-1.5" aria-label="Importer depuis un autre navigateur">
      {sources.map((source) => {
        const imported = done.includes(source.id)
        return (
          <div key={source.id} className="flex items-center justify-between gap-3 rounded-row bg-field px-3 py-2
            shadow-field">
            <span className="text-[12.5px] text-ink">{source.name}</span>
            <PushButton tone={imported ? 'neutral' : 'guard'} disabled={imported}
              onClick={() => {
                send({ kind: 'importBrowser', id: source.id })
                setDone([...done, source.id])
              }}>
              {imported ? 'Importé' : 'Importer favoris et historique'}
            </PushButton>
          </div>
        )
      })}
    </div>
  )
}
