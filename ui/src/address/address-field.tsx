// Responsabilite : champ d'adresse compact — l'hote seul au repos, l'URL complete a la saisie,
// le zoom en badge quand il s'ecarte de 100 %, la progression en trait au bas.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { TabView, UiRequest } from '../shared/contract'
import { formatZoom } from '../shared/format'
import { QUICK } from '../shared/design/motion'
import { readUrl } from '../shared/url-shape'
import { LoadProgress } from './load-progress'
import { IconSearch, IconStar, IconStarFilled } from '../shared/design/icons'
import { SecurityMark } from './security-mark'
import { SuggestionList } from './suggestion-list'
import { useSuggestions, type Suggestions } from './use-suggestions'
import type { CoreState } from '../shared/core-state'
import { useAddressField, type AddressFieldState } from './use-address-field'

export interface AddressFieldProps {
  tab: TabView | null
  onSubmit: (input: string) => void
  onResetZoom: () => void
  /** Clic sur le cadenas : la securite du site. */
  onOpenSecurity?: () => void
  /** Suggestions recues du coeur, et l'envoi des demandes. */
  suggestions: CoreState['suggestions']
  send: (request: UiRequest) => void
  /** Incremente pour donner le focus au champ depuis l'exterieur. */
  focusToken: number
  /** La page est-elle dans les favoris ? `null` : page interne, pas d'etoile. */
  bookmarked: boolean | null
  onToggleBookmark: () => void
  /** Echap dans l'adresse. */
  onLeave?: () => void
}

function BookmarkStar({ on, onToggle }: { on: boolean; onToggle: () => void }): ReactElement {
  const label = on ? 'Retirer des favoris' : 'Ajouter aux favoris (Ctrl+D)'
  return (
    <button type="button" title={label} aria-label={label} aria-pressed={on}
      onPointerDown={(event) => event.stopPropagation()} onClick={onToggle}
      className={`z-10 -mr-1 grid shrink-0 place-items-center rounded-full p-1 transition-colors duration-100
        ${on ? 'text-warn' : 'text-ink-faint hover:text-ink'}`}>
      {on ? <IconStarFilled size={13} /> : <IconStar size={13} />}
    </button>
  )
}

/** Comme Chrome : une page non chiffree ou au certificat refuse le dit en toutes lettres, avant l'adresse. */
function UnsafeLabel({ security }: { security: string }): ReactElement | null {
  if (security !== 'insecure' && security !== 'invalid') return null
  return (
    <span data-unsafe={security}
      className={`mr-1.5 shrink-0 font-medium ${security === 'invalid' ? 'text-danger' : 'text-warn'}`}>
      Non sécurisé
    </span>
  )
}

function RestingHost({ url, security }: { url: string; security: string }): ReactElement {
  const shape = readUrl(url)
  if (shape.host.length === 0) {
    return <span className="truncate whitespace-nowrap text-ink-faint">Rechercher ou saisir une URL</span>
  }
  return (
    <span className="flex min-w-0 items-center">
      <UnsafeLabel security={security} />
      <span className="truncate">
      <span className="text-ink">{shape.host}</span>
      {shape.path.length > 0 && <span className="text-ink-faint">{shape.path}</span>}
      </span>
    </span>
  )
}

function ZoomBadge({ zoom, onReset }: { zoom: number; onReset: () => void }): ReactElement {
  return (
    <AnimatePresence>
      {Math.abs(zoom - 1) > 0.001 && (
        <motion.button
          key="zoom"
          type="button"
          title="Revenir à 100 %"
          aria-label={`Zoom ${formatZoom(zoom)}, revenir à 100 %`}
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.9 }}
          transition={QUICK}
          onPointerDown={(event) => event.stopPropagation()}
          onClick={onReset}
          className="numerique z-10 shrink-0 rounded bg-ink/10 px-1 py-px text-[10.5px] text-ink-muted
            transition-colors duration-100 hover:bg-ink/15 hover:text-ink"
        >
          {formatZoom(zoom)}
        </motion.button>
      )}
    </AnimatePresence>
  )
}

function AddressInput({ field, list }: { field: AddressFieldState; list: Suggestions }): ReactElement {
  return (
    <input
      ref={field.inputRef}
      value={field.value}
      spellCheck={false}
      autoComplete="off"
      aria-label="Adresse"
      placeholder={field.editing ? 'Rechercher ou saisir une URL' : undefined}
      onChange={(event) => field.onChange(event.target.value)}
      onFocus={field.onFocus}
      onBlur={field.onBlur}
      onKeyDown={(event) => {
        if (!list.onKey(event)) field.onKeyDown(event)
      }}
      className={`min-w-0 flex-1 bg-transparent text-[12.5px] outline-none select-text
        placeholder:text-ink-faint ${field.editing ? 'text-ink' : 'text-transparent'}`}
    />
  )
}

export function AddressField(props: AddressFieldProps): ReactElement {
  const { tab, onSubmit, onResetZoom, focusToken, onOpenSecurity, send } = props
  const field = useAddressField(tab, onSubmit, focusToken, props.onLeave)
  const list = useSuggestions(field.value, field.editing, props.suggestions, send, onSubmit,
    () => field.inputRef.current?.blur())
  const security = tab === null || tab.url.length === 0 || tab.url === 'about:blank' ? 'blank' : tab.security
  return (
    <div className="relative">
      <div
        className={`relative flex h-9 min-w-0 items-center gap-2 overflow-hidden rounded-full bg-field px-3.5
          shadow-field transition-colors duration-100 ${field.editing ? 'ring-1 ring-guard/60' : 'hover:bg-hover'}`}
      >
        {field.editing
          ? <IconSearch size={13} className="shrink-0 text-ink-faint" aria-hidden />
          : <SecurityMark security={security} onOpen={onOpenSecurity} />}
        <AddressInput field={field} list={list} />
        {!field.editing && (
          <div className={`pointer-events-none absolute inset-y-0 left-[39px] flex items-center
            ${tab !== null && Math.abs(tab.zoom - 1) > 0.001 ? 'right-[80px]' : 'right-[34px]'}
            overflow-hidden text-[12.5px] leading-none`}>
            <RestingHost url={tab?.url ?? ''} security={security} />
          </div>
        )}
        {!field.editing && tab !== null && <ZoomBadge zoom={tab.zoom} onReset={onResetZoom} />}
        {!field.editing && props.bookmarked !== null && (
          <BookmarkStar on={props.bookmarked} onToggle={props.onToggleBookmark} />
        )}
        <LoadProgress loading={tab?.loading ?? false} progress={tab?.progress ?? 0} />
      </div>
      <SuggestionList items={list.items} selected={list.selected} onPick={list.pick} />
    </div>
  )
}
