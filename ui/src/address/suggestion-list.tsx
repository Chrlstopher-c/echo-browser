// Responsabilite : la liste de suggestions sous l'adresse — onglets ouverts, favoris, historique ; la ligne choisie au
// clavier est mise en relief. Un clic choisit sans faire perdre le focus au champ.

import type { ReactElement } from 'react'
import type { SuggestionView } from '../shared/contract'
import { IconClock, IconGlobe, IconSearch, IconStar } from '../shared/design/icons'

const KIND = {
  search: { label: 'Rechercher', Icon: IconSearch },
  query: { label: 'Suggestion de recherche', Icon: IconSearch },
  tab: { label: 'Onglet ouvert', Icon: IconGlobe },
  bookmark: { label: 'Favori', Icon: IconStar },
  history: { label: 'Historique', Icon: IconClock },
} as const

export interface SuggestionListProps {
  items: SuggestionView[]
  selected: number
  onPick: (item: SuggestionView) => void
}

function hostOf(url: string): string {
  return url.replace(/^[a-z]+:\/\//, '').replace(/\/$/, '')
}

/** Moteur d'une adresse de recherche, lisible : « google.com ». */
function engineOf(url: string): string {
  try {
    return new URL(url).hostname.replace(/^(www|search)\./, '')
  } catch {
    return ''
  }
}

function Lines({ item }: { item: SuggestionView }): ReactElement {
  if (item.kind === 'search') {
    return (
      <>
        <p className="truncate text-[12px] text-ink">Rechercher « {item.title} »</p>
        <p className="truncate text-[10.5px] text-ink-faint">sur {engineOf(item.url)}</p>
      </>
    )
  }
  if (item.kind === 'query') return <p className="truncate text-[12px] text-ink">{item.title}</p>
  return (
    <>
      <p className="truncate text-[12px] text-ink">{item.title || hostOf(item.url)}</p>
      <p className="numerique truncate text-[10.5px] text-ink-faint">{hostOf(item.url)}</p>
    </>
  )
}

export function SuggestionList({ items, selected, onPick }: SuggestionListProps): ReactElement | null {
  if (items.length === 0) return null
  return (
    <ul role="listbox" aria-label="Suggestions"
      className="absolute inset-x-0 top-[calc(100%+6px)] z-30 flex max-h-[60vh] flex-col overflow-y-auto rounded-tile
        bg-card p-1 shadow-lift">
      {items.map((item, index) => {
        const { label, Icon } = KIND[item.kind]
        return (
          <li key={`${item.kind}-${item.url}-${item.title}`} role="option" aria-selected={index === selected} data-kind={item.kind}
            title={label}
            onMouseDown={(event) => { event.preventDefault(); onPick(item) }}
            className={`flex cursor-default items-center gap-2 rounded-row px-2 py-1.5
              ${index === selected ? 'bg-hover shadow-pressed' : 'hover:bg-hover'}`}>
            <Icon size={13} className="shrink-0 text-ink-faint" aria-label={label} />
            <div className="min-w-0 flex-1"><Lines item={item} /></div>
            {item.kind === 'tab' && <span className="shrink-0 text-[10.5px] text-guard">aller</span>}
          </li>
        )
      })}
    </ul>
  )
}
