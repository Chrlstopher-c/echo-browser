// Responsabilite : une liste de sites en pastilles (sites qui ne dorment jamais…) — retirer d'un clic, ajouter en
// tapant le site puis Entree. Le reglage reste une chaine separee par des virgules.

import { useState, type ReactElement } from 'react'
import { IconClose, IconPlus } from '../shared/design/icons'

export interface SiteListProps {
  value: string
  label: string
  onChange: (next: string) => void
}

/** « https://www.exemple.fr/page » → « exemple.fr ». */
function siteOf(input: string): string {
  return input.trim().toLowerCase().replace(/^[a-z]+:\/\//, '').replace(/^www\./, '').split('/')[0] ?? ''
}

export function SiteList({ value, label, onChange }: SiteListProps): ReactElement {
  const sites = value.split(',').map((s) => s.trim()).filter(Boolean)
  const [draft, setDraft] = useState('')
  const add = (): void => {
    const site = siteOf(draft)
    if (site.length > 0 && !sites.includes(site)) onChange([...sites, site].join(','))
    setDraft('')
  }
  return (
    <div className="flex flex-col gap-2" aria-label={label}>
      <div className="flex flex-wrap gap-1.5">
        {sites.map((site) => (
          <span key={site} className="flex items-center gap-1 rounded-full bg-card py-0.5 pr-1 pl-2.5 text-[11.5px]
            text-ink shadow-card">
            {site}
            <button type="button" aria-label={`Retirer ${site}`} title={`Retirer ${site}`}
              onClick={() => onChange(sites.filter((s) => s !== site).join(','))}
              className="grid size-5 place-items-center rounded-full text-ink-faint hover:bg-hover hover:text-ink">
              <IconClose size={10} />
            </button>
          </span>
        ))}
      </div>
      <div className="flex h-8 items-center gap-1.5 rounded-row bg-field pr-1 pl-2.5 shadow-field
        focus-within:ring-1 focus-within:ring-guard/60">
        <input value={draft} placeholder="Ajouter un site (ex. : web.whatsapp.com)" aria-label={`Ajouter à ${label}`}
          spellCheck={false} autoComplete="off" onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter') add() }}
          className="min-w-0 flex-1 bg-transparent text-[12px] text-ink outline-none placeholder:text-ink-faint" />
        <button type="button" aria-label="Ajouter ce site" onClick={add}
          className="grid size-6 place-items-center rounded-row text-ink-muted hover:bg-hover hover:text-ink">
          <IconPlus size={13} />
        </button>
      </div>
    </div>
  )
}
