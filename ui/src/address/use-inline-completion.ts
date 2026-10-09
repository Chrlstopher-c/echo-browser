// Responsabilite : complete l'adresse pendant la frappe, comme Chrome — « git » devient « github.com », la partie
// ajoutee reste selectionnee (une touche de plus la remplace, Entree la valide). Seulement quand l'utilisateur ajoute
// des caracteres, jamais quand il efface.

import { useEffect, useRef, useState } from 'react'
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
    field.onChange(text + target.slice(text.length))
    requestAnimationFrame(() => field.inputRef.current?.setSelectionRange(text.length, target.length))
  }, [received, typed, field])
  return { typed, onChange }
}
