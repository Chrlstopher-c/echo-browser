// Responsabilite : complete l'adresse pendant la frappe, comme Chrome — « git » devient « github.com », la partie
// ajoutee reste selectionnee (une touche de plus la remplace, Entree la valide). Seulement quand l'utilisateur ajoute
// des caracteres, jamais quand il efface.

import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import type { CoreState } from '../shared/core-state'
import type { AddressFieldState } from './use-address-field'

/** L'adresse telle qu'on la tape : sans schema, sans « www. », sans barre finale. */
function bare(url: string): string {
  return url.replace(/^https?:\/\//, '').replace(/^www\./, '').replace(/\/$/, '')
}

export interface InlineCompletion {
  /** Ce que l'utilisateur a reellement tape (la requete des suggestions). */
  typed: string
  onChange: (next: string) => void
}

export function useInlineCompletion(field: AddressFieldState, received: CoreState['suggestions']): InlineCompletion {
  const [typed, setTyped] = useState('')
  const growing = useRef(false)
  // Selection a poser une fois la valeur completee rendue (React replace le curseur en fin de champ a chaque rendu).
  const pending = useRef<[number, number] | null>(null)
  useLayoutEffect(() => {
    const range = pending.current
    if (range === null) return
    pending.current = null
    field.inputRef.current?.setSelectionRange(range[0], range[1])
  }, [field.value, field.inputRef])
  const onChange = (next: string): void => {
    growing.current = next.length > typed.length && next.startsWith(typed)
    setTyped(next)
    field.onChange(next)
  }
  useEffect(() => {
    if (!field.editing) setTyped('')
  }, [field.editing])
  useEffect(() => {
    const text = typed.trim()
    if (!growing.current || !field.editing || received?.query !== text || text.length < 2 || /\s/.test(text)) return
    const lower = text.toLowerCase()
    const hit = received.items.find((item) => item.kind !== 'search' && item.kind !== 'query'
      && bare(item.url).toLowerCase().startsWith(lower))
    if (hit === undefined) return
    const full = bare(hit.url)
    // Sans « / » tape, on complete jusqu'au site seulement (comme Chrome) ; sinon jusqu'a la page.
    const target = text.includes('/') ? full : full.split('/')[0] ?? full
    if (target.length <= text.length) return
    growing.current = false
    pending.current = [text.length, target.length]
    field.onChange(text + target.slice(text.length))
  }, [received, typed, field])
  return { typed, onChange }
}
