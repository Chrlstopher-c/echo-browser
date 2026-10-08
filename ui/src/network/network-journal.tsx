// Responsabilite : journal d'acces du site de l'onglet actif — premiers contacts avec des tiers, permissions demandees
// et decidees, telechargements — le plus recent d'abord.

import type { ReactElement } from 'react'
import type { JournalEntryView } from '../shared/contract'
import { IconDownload, IconGlobe, IconLock } from '../shared/design/icons'

const PERMISSION: Record<string, string> = {
  position: 'Position', camera: 'Caméra', microphone: 'Micro', notifications: 'Notifications',
  'presse-papiers': 'Presse-papiers', ecran: 'Partage d’écran',
}

function describe(entry: JournalEntryView): { icon: ReactElement; text: string } {
  if (entry.kind === 'tiers') return { icon: <IconGlobe size={12} />, text: `Premier contact avec ${entry.detail}` }
  if (entry.kind === 'telechargement') {
    return { icon: <IconDownload size={12} />, text: `Téléchargement : ${entry.detail}` }
  }
  const [kind = '', verdict = ''] = entry.detail.split(' : ')
  return { icon: <IconLock size={12} />, text: `${PERMISSION[kind] ?? kind} demandé — ${verdict}` }
}

function when(seconds: number): string {
  return new Date(seconds * 1000).toLocaleString('fr-FR', { day: 'numeric', month: 'short', hour: '2-digit',
    minute: '2-digit' })
}

export function NetworkJournal({ journal }: { journal: JournalEntryView[] }): ReactElement {
  if (journal.length === 0) return <p className="px-2 text-[11.5px] text-ink-faint">Rien de noté pour ce site.</p>
  return (
    <div className="flex flex-col">
      {journal.map((entry, i) => {
        const { icon, text } = describe(entry)
        return (
          <div key={`${entry.at}-${i}`} className="flex items-start gap-2 px-2 py-1">
            <span className="mt-0.5 shrink-0 text-ink-faint">{icon}</span>
            <div className="min-w-0">
              <p className="text-[11.5px] leading-snug text-ink">{text}</p>
              <p className="numerique text-[10px] text-ink-faint">{when(entry.at)}</p>
            </div>
          </div>
        )
      })}
    </div>
  )
}
