// Responsabilite : champ d'adresse compact — l'hote seul au repos, l'URL complete a la saisie.

import type { ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import { readUrl } from '../shared/url-shape'
import { LoadProgress } from './load-progress'
import { SecurityMark } from './security-mark'
import { useAddressField } from './use-address-field'

export interface AddressFieldProps {
  tab: TabView | null
  onSubmit: (input: string) => void
  /** Incremente pour donner le focus au champ depuis l'exterieur. */
  focusToken: number
}

function RestingHost({ url }: { url: string }): ReactElement {
  const shape = readUrl(url)
  if (shape.host.length === 0) {
    return <span className="text-ink-faint">Rechercher ou saisir une adresse</span>
  }
  return <span className="truncate text-ink">{shape.host}</span>
}

export function AddressField({ tab, onSubmit, focusToken }: AddressFieldProps): ReactElement {
  const field = useAddressField(tab, onSubmit, focusToken)
  const security = tab === null || tab.url.length === 0 ? 'blank' : tab.security
  return (
    <div
      className={`relative flex h-8 min-w-0 items-center gap-2 overflow-hidden rounded-row bg-field px-2.5
        shadow-card transition-colors duration-100 ${field.editing ? 'ring-1 ring-guard/60' : 'hover:bg-hover'}`}
    >
      <SecurityMark security={security} />
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
        <div className="pointer-events-none absolute inset-y-0 right-2.5 left-[31px] flex items-center
          overflow-hidden text-[12.5px]">
          <RestingHost url={tab?.url ?? ''} />
        </div>
      )}
      <LoadProgress loading={tab?.loading ?? false} progress={tab?.progress ?? 0} />
    </div>
  )
}
