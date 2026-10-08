// Responsabilite : Bibliotheque → Machines — les onglets ouverts sur les autres machines du compte Echo, a rouvrir ici.

import type { ReactElement } from 'react'
import type { RemoteMachineView } from '../../shared/contract'
import { ListRow } from '../../shared/design/list-row'
import { SectionLabel } from '../../shared/design/section-label'
import { SiteMark } from '../../shared/design/site-mark'

function ago(seconds: number): string {
  const minutes = Math.max(0, Math.round((Date.now() / 1000 - seconds) / 60))
  if (minutes < 1) return 'à l’instant'
  if (minutes < 60) return `il y a ${minutes} min`
  if (minutes < 48 * 60) return `il y a ${Math.round(minutes / 60)} h`
  return `il y a ${Math.round(minutes / 1440)} j`
}

export interface DevicesPanelProps {
  machines: RemoteMachineView[]
  onOpen: (url: string) => void
}

export function DevicesPanel({ machines, onOpen }: DevicesPanelProps): ReactElement {
  return (
    <div className="flex flex-col gap-3">
      {machines.map((machine) => (
        <section key={machine.name + machine.updated}>
          <SectionLabel aside={<span className="text-[10.5px] text-ink-faint">{ago(machine.updated)}</span>}>
            {machine.name}
          </SectionLabel>
          {machine.tabs.map((tab) => (
            <ListRow key={tab.url} onClick={() => onOpen(tab.url)}>
              <SiteMark url={tab.url} favicon={null} size={14} />
              <span className="min-w-0 flex-1 truncate text-ink">{tab.title || tab.url}</span>
            </ListRow>
          ))}
        </section>
      ))}
    </div>
  )
}
