// Responsabilite : Bibliotheque → Routines — les suites de sites enregistrees ; les ouvrir d'un geste, les retirer.

import type { ReactElement } from 'react'
import type { RoutineView, UiRequest } from '../shared/contract'
import { EmptyState } from '../shared/design/empty-state'
import { IconSparkle, IconTrash } from '../shared/design/icons'
import { IconButton } from '../shared/design/icon-button'
import { PushButton } from '../shared/design/push-button'

function hostOf(url: string): string {
  return url.replace(/^[a-z]+:\/\//, '').split('/')[0] ?? url
}

export function RoutinesPanel({ routines, send }: { routines: RoutineView[]; send: (r: UiRequest) => void }):
  ReactElement {
  if (routines.length === 0) {
    return <EmptyState icon={<IconSparkle size={18} />} title="Aucune routine"
      hint="Quand vous ouvrez souvent les mêmes sites à la suite, Echo propose d’en faire une routine." />
  }
  return (
    <div className="flex flex-col gap-1">
      {routines.map((r) => (
        <div key={r.id} className="group flex items-center gap-2 rounded-row px-2 py-1.5 hover:bg-hover">
          <div className="min-w-0 flex-1">
            <p className="truncate text-[12.5px] text-ink">{r.name}</p>
            <p className="truncate text-[10.5px] text-ink-faint">{r.urls.map(hostOf).join(' → ')}</p>
          </div>
          <PushButton tone="guard" onClick={() => send({ kind: 'routineOpen', id: r.id })}>Ouvrir</PushButton>
          <IconButton label="Retirer la routine" onClick={() => send({ kind: 'routineRemove', id: r.id })}>
            <IconTrash size={13} />
          </IconButton>
        </div>
      ))}
    </div>
  )
}
