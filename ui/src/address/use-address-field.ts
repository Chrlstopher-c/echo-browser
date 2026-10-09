// Responsabilite : etat du champ d'adresse — brouillon de saisie, focus, selection, validation.

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
  /** Echap : l'utilisateur renonce (fenetre etroite : la barre repart). */
  onLeave?: () => void,
): AddressFieldState {
  // Un nouvel onglet (page d'Echo) s'ouvre sur une adresse vide, prete a taper : jamais son adresse interne.
  const url = tab === null || tab.url === 'about:blank' || tab.url.startsWith('echo://') ? '' : tab.url
  const [draft, setDraft] = useState(url)
  const [editing, setEditing] = useState(false)
  const inputRef = useRef<HTMLInputElement | null>(null)

  useEffect(() => {
    if (!editing) setDraft(url)
  }, [url, editing])

  // Ctrl+L, ou le rail : focus et contenu selectionne, pour retaper par-dessus.
  useEffect(() => {
    if (focusToken === 0) return
    const input = inputRef.current
    if (input === null) return
    input.focus()
    requestAnimationFrame(() => input.select())
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
      if (event.key === 'Escape') {
        event.stopPropagation()
        inputRef.current?.blur()
        onLeave?.()
      }
    },
    [draft, onSubmit, onLeave],
  )

  return { value: draft, editing, inputRef, onChange: setDraft, onFocus, onBlur, onKeyDown }
}
