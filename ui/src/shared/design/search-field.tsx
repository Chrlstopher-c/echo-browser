// Responsabilite : champ de recherche d'une feuille — loupe, saisie, effacement.

import type { ReactElement } from 'react'
import { IconClose, IconSearch } from './icons'

export interface SearchFieldProps {
  value: string
  placeholder: string
  onChange: (next: string) => void
  autoFocus?: boolean
}

export function SearchField({ value, placeholder, onChange, autoFocus = false }: SearchFieldProps): ReactElement {
  return (
    <div className="flex h-8 items-center gap-2 rounded-row bg-field pr-1 pl-2.5 shadow-field
      focus-within:ring-1 focus-within:ring-guard/60">
      <IconSearch size={13} className="shrink-0 text-ink-faint" />
      <input
        value={value}
        autoFocus={autoFocus}
        spellCheck={false}
        autoComplete="off"
        aria-label={placeholder}
        placeholder={placeholder}
        onChange={(event) => onChange(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === 'Escape' && value.length > 0) {
            event.stopPropagation()
            onChange('')
          }
        }}
        className="min-w-0 flex-1 bg-transparent text-[12px] text-ink outline-none select-text
          placeholder:text-ink-faint"
      />
      {value.length > 0 && (
        <button
          type="button"
          aria-label="Effacer la recherche"
          onClick={() => onChange('')}
          className="grid size-6 shrink-0 place-items-center rounded-md text-ink-faint hover:bg-hover hover:text-ink"
        >
          <IconClose size={12} />
        </button>
      )}
    </div>
  )
}
