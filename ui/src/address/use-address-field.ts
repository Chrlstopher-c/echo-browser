// Responsabilite : etat du champ d'adresse — brouillon de saisie, focus, validation.

import { useCallback, useEffect, useRef, useState } from 'react'
import type { KeyboardEvent, RefObject } from 'react'
import type { TabView } from '../shared/contract'

export interface AddressFieldState {
  value: string
  editing: boolean
  inputRef: RefObject<HTMLInputElement | null>
  onChange: (next: string) => void
  onFocus: () => void
  onBlur: () => void
  onKeyDown: (event: KeyboardEvent<HTMLInputElement>) => void
}

export function useAddressField(
  tab: TabView | null,
  onSubmit: (input: string) => void,
  focusToken: number,
): AddressFieldState {
  const url = tab?.url ?? ''
  const [draft, setDraft] = useState(url)
  const [editing, setEditing] = useState(false)
  const inputRef = useRef<HTMLInputElement | null>(null)

  useEffect(() => {
    if (!editing) setDraft(url)
  }, [url, editing])

  useEffect(() => {
    if (focusToken > 0) inputRef.current?.focus()
  }, [focusToken])

  const onFocus = useCallback((): void => {
    setEditing(true)
    requestAnimationFrame(() => inputRef.current?.select())
  }, [])

  const onBlur = useCallback((): void => {
    setEditing(false)
    setDraft(url)
  }, [url])

  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLInputElement>): void => {
      if (event.key === 'Enter' && draft.trim().length > 0) {
        onSubmit(draft.trim())
        inputRef.current?.blur()
      }
      if (event.key === 'Escape') inputRef.current?.blur()
    },
    [draft, onSubmit],
  )

  return { value: draft, editing, inputRef, onChange: setDraft, onFocus, onBlur, onKeyDown }
}
