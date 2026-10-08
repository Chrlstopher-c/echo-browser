// Responsabilite : barre de recherche dans la page, sous l'adresse — champ, « 3/12 », precedent, suivant, fermer.

import { useEffect, useRef, type ReactElement } from 'react'
import { IconButton } from '../shared/design/icon-button'
import { IconBack, IconClose, IconForward, IconSearch } from '../shared/design/icons'
import type { FindController } from './use-find'

export interface FindBarProps {
  find: FindController
  result: { count: number; current: number }
}

function Counter({ find, result }: FindBarProps): ReactElement | null {
  if (find.text.length === 0) return null
  const none = result.count === 0
  return (
    <span aria-live="polite" className={`numerique shrink-0 text-[11px] ${none ? 'text-danger' : 'text-ink-muted'}`}>
      {none ? 'Aucun' : `${result.current}/${result.count}`}
    </span>
  )
}

export function FindBar({ find, result }: FindBarProps): ReactElement | null {
  const input = useRef<HTMLInputElement>(null)
  useEffect(() => {
    if (!find.open) return
    input.current?.focus()
    input.current?.select()
  }, [find.open, find.focusToken])
  if (!find.open) return null
  return (
    <div role="search" className="flex h-9 items-center gap-1.5 rounded-full bg-field pr-1 pl-3 shadow-field">
      <IconSearch size={13} className="shrink-0 text-ink-faint" aria-hidden />
      <input ref={input} value={find.text} aria-label="Rechercher dans la page" placeholder="Rechercher dans la page"
        spellCheck={false} autoComplete="off"
        onChange={(event) => find.setText(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === 'Enter') {
            event.preventDefault()
            find.step(!event.shiftKey)
          } else if (event.key === 'Escape') {
            event.preventDefault()
            find.close()
          }
        }}
        className="min-w-0 flex-1 bg-transparent text-[12.5px] text-ink outline-none placeholder:text-ink-faint" />
      <Counter find={find} result={result} />
      <IconButton label="Occurrence précédente (Maj+Entrée)" onClick={() => find.step(false)}>
        <IconBack size={13} className="rotate-90" />
      </IconButton>
      <IconButton label="Occurrence suivante (Entrée)" onClick={() => find.step(true)}>
        <IconForward size={13} className="rotate-90" />
      </IconButton>
      <IconButton label="Fermer la recherche (Échap)" onClick={find.close}>
        <IconClose size={13} />
      </IconButton>
    </div>
  )
}
