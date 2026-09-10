// Responsabilite : champ d'adresse compact — l'hote seul au repos, l'URL complete a la saisie,
// le zoom en badge quand il s'ecarte de 100 %, la progression en trait au bas.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import { formatZoom } from '../shared/format'
import { QUICK } from '../shared/design/motion'
import { readUrl } from '../shared/url-shape'
import { LoadProgress } from './load-progress'
import { SecurityMark } from './security-mark'
import { useAddressField } from './use-address-field'

export interface AddressFieldProps {
  tab: TabView | null
  onSubmit: (input: string) => void
  onResetZoom: () => void
  /** Incremente pour donner le focus au champ depuis l'exterieur. */
  focusToken: number
}

function RestingHost({ url }: { url: string }): ReactElement {
  const shape = readUrl(url)
  if (shape.host.length === 0) {
    return <span className="text-ink-faint">Rechercher ou saisir une adresse</span>
  }
  return (
    <span className="truncate">
      <span className="text-ink">{shape.host}</span>
      {shape.path.length > 0 && <span className="text-ink-faint">{shape.path}</span>}
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
          className="numerique z-10 shrink-0 rounded bg-ink/10 px-1 py-px text-[10px] text-ink-muted
            transition-colors duration-100 hover:bg-ink/15 hover:text-ink"
        >
          {formatZoom(zoom)}
        </motion.button>
      )}
    </AnimatePresence>
  )
}

export function AddressField({ tab, onSubmit, onResetZoom, focusToken }: AddressFieldProps): ReactElement {
  const field = useAddressField(tab, onSubmit, focusToken)
  const security = tab === null || tab.url.length === 0 || tab.url === 'about:blank' ? 'blank' : tab.security
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
        placeholder={field.editing ? 'Rechercher ou saisir une adresse' : undefined}
        onChange={(event) => field.onChange(event.target.value)}
        onFocus={field.onFocus}
        onBlur={field.onBlur}
        onKeyDown={field.onKeyDown}
        className={`numerique min-w-0 flex-1 bg-transparent text-[12px] outline-none select-text
          placeholder:text-ink-faint ${field.editing ? 'text-ink' : 'text-transparent'}`}
      />
      {!field.editing && (
        <div className="pointer-events-none absolute inset-y-0 right-2.5 left-[31px] flex items-center
          overflow-hidden text-[12.5px]">
          <RestingHost url={tab?.url ?? ''} />
        </div>
      )}
      {!field.editing && tab !== null && <ZoomBadge zoom={tab.zoom} onReset={onResetZoom} />}
      <LoadProgress loading={tab?.loading ?? false} progress={tab?.progress ?? 0} />
    </div>
  )
}
