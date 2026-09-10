// Responsabilite : champ d'adresse — saisie brute au focus, lecture hierarchisee au repos.

import type { ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import { readUrl } from '../shared/url-shape'
import { SecurityMark } from './security-mark'
import { useAddressField } from './use-address-field'

export interface AddressFieldProps {
  tab: TabView | null
  onSubmit: (input: string) => void
}

function PrettyUrl({ url }: { url: string }): ReactElement {
  const shape = readUrl(url)
  if (shape.host.length === 0) {
    return <span className="text-ink-faint">Rechercher ou saisir une adresse</span>
  }
  return (
    <span className="truncate">
      <span className="text-ink">{shape.host}</span>
      <span className="text-ink-faint">{shape.path}</span>
    </span>
  )
}

export function AddressField({ tab, onSubmit }: AddressFieldProps): ReactElement {
  const field = useAddressField(tab, onSubmit)
  const safety = readUrl(tab?.url ?? '').safety
  return (
    <div
      className={`relative flex h-7 min-w-0 flex-1 items-center gap-2 rounded-field border px-2.5
        transition-colors duration-100
        ${field.editing ? 'border-guard/70 bg-inset' : 'border-hairline bg-raised hover:border-edge'}`}
    >
      <SecurityMark safety={safety} />
      <input
        ref={field.inputRef}
        value={field.value}
        spellCheck={false}
        autoComplete="off"
        aria-label="Adresse"
        onChange={(event) => field.onChange(event.target.value)}
        onFocus={field.onFocus}
        onBlur={field.onBlur}
        onKeyDown={field.onKeyDown}
        className={`min-w-0 flex-1 bg-transparent text-[12.5px] outline-none select-text
          ${field.editing ? 'text-ink' : 'text-transparent'}`}
      />
      {!field.editing && (
        <div className="pointer-events-none absolute inset-y-0 left-[30px] right-2.5 flex items-center
          overflow-hidden text-[12.5px]">
          <PrettyUrl url={tab?.url ?? ''} />
        </div>
      )}
    </div>
  )
}
