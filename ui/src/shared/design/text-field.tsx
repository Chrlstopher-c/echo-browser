// Responsabilite : champ texte d'un reglage — modifie en place, valide sur Entree ou a la perte du focus,
// abandonne sur Echap. La valeur affichee suit le coeur tant qu'on ne tape pas.

import { useCallback, useEffect, useState, type KeyboardEvent, type ReactElement } from 'react'

export interface TextFieldProps {
  value: string
  label: string
  placeholder?: string
  mono?: boolean
  onCommit: (next: string) => void
}

interface Draft {
  draft: string
  setDraft: (next: string) => void
  begin: () => void
  commit: () => void
  abandon: () => void
}

function useDraft(value: string, onCommit: (next: string) => void): Draft {
  const [draft, setDraft] = useState(value)
  const [editing, setEditing] = useState(false)
  useEffect(() => {
    if (!editing) setDraft(value)
  }, [value, editing])
  const begin = useCallback((): void => setEditing(true), [])
  const commit = useCallback((): void => {
    setEditing(false)
    const next = draft.trim()
    if (next !== value) onCommit(next)
  }, [draft, value, onCommit])
  const abandon = useCallback((): void => {
    setDraft(value)
    setEditing(false)
  }, [value])
  return { draft, setDraft, begin, commit, abandon }
}

export function TextField({ value, label, placeholder = '', mono = false, onCommit }: TextFieldProps): ReactElement {
  const field = useDraft(value, onCommit)
  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>): void => {
    if (event.key === 'Enter') event.currentTarget.blur()
    if (event.key === 'Escape') {
      event.stopPropagation()
      field.abandon()
      event.currentTarget.blur()
    }
  }
  return (
    <input
      value={field.draft}
      aria-label={label}
      placeholder={placeholder}
      spellCheck={false}
      autoComplete="off"
      onFocus={field.begin}
      onChange={(event) => field.setDraft(event.target.value)}
      onBlur={field.commit}
      onKeyDown={onKeyDown}
      className={`h-7 w-full min-w-0 rounded-row bg-field px-2 text-[11.5px] text-ink shadow-card outline-none
        select-text placeholder:text-ink-faint focus:ring-1 focus:ring-guard/60 ${mono ? 'numerique' : ''}`}
    />
  )
}
