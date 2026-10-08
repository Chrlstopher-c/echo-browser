// Responsabilite : etat de la recherche dans la page — ouverture (Ctrl+F, clic droit), texte, envoi au coeur, fermeture.

import { useCallback, useEffect, useRef, useState } from 'react'
import type { UiRequest } from '../shared/contract'

export interface FindController {
  open: boolean
  text: string
  /** Incremente quand le champ doit reprendre le focus. */
  focusToken: number
  setText: (text: string) => void
  step: (forward: boolean) => void
  close: () => void
}

export function useFind(send: (request: UiRequest) => void, requested: number, activeId: number | null): FindController {
  const [open, setOpen] = useState(false)
  const [text, setTextState] = useState('')
  const [focusToken, setFocusToken] = useState(0)
  const first = useRef(true)
  useEffect(() => {
    if (first.current) {
      first.current = false
      return
    }
    setOpen(true)
    setFocusToken((n) => n + 1)
  }, [requested])
  const close = useCallback((): void => {
    send({ kind: 'stopFind' })
    setOpen(false)
  }, [send])
  // Un autre onglet : la recherche precedente ne le concerne pas.
  useEffect(() => {
    setOpen(false)
  }, [activeId])
  const setText = useCallback((next: string): void => {
    setTextState(next)
    send({ kind: 'find', text: next, forward: true, next: false })
  }, [send])
  const step = useCallback((forward: boolean): void => {
    if (text.length > 0) send({ kind: 'find', text, forward, next: true })
  }, [send, text])
  return { open, text, focusToken, setText, step, close }
}
