// Responsabilite : suggestions de l'adresse pendant la saisie — demande au coeur (apres une courte pause de frappe),
// choix au clavier (fleches, Entree), fermeture (Echap, perte du focus).

import { useEffect, useState, type KeyboardEvent } from 'react'
import type { SuggestionView, UiRequest } from '../shared/contract'
import type { CoreState } from '../shared/core-state'

const TYPING_MS = 80

export interface Suggestions {
  items: SuggestionView[]
  selected: number
  /** Traite une touche ; vrai si elle a ete consommee par la liste. */
  onKey: (event: KeyboardEvent<HTMLInputElement>) => boolean
  pick: (item: SuggestionView) => void
}

export function useSuggestions(
  query: string,
  editing: boolean,
  received: CoreState['suggestions'],
  send: (request: UiRequest) => void,
  /** Valide une adresse comme si elle avait ete tapee. */
  submit: (url: string) => void,
  done: () => void,
): Suggestions {
  const [selected, setSelected] = useState(-1)
  // Ferme la liste des le choix, sans attendre la perte du focus.
  const [pickedFor, setPickedFor] = useState<string | null>(null)
  const typed = query.trim()
  useEffect(() => {
    setSelected(-1)
    if (!editing || typed.length === 0) return
    const wait = setTimeout(() => send({ kind: 'suggest', query: typed }), TYPING_MS)
    return () => clearTimeout(wait)
  }, [typed, editing, send])
  const open = editing && typed.length > 0 && pickedFor !== typed && received?.query === typed
  const items = open ? received.items.slice(0, 9) : []
  const pick = (item: SuggestionView): void => {
    if (item.tab !== null) send({ kind: 'selectTab', id: item.tab })
    else submit(item.url)
    setPickedFor(typed)
    done()
  }
  const onKey = (event: KeyboardEvent<HTMLInputElement>): boolean => {
    if (items.length === 0) return false
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault()
      setSelected((current) =>
        event.key === 'ArrowDown' ? Math.min(current + 1, items.length - 1) : Math.max(current - 1, -1))
      return true
    }
    const chosen = items[selected]
    if (event.key === 'Enter' && chosen !== undefined) {
      event.preventDefault()
      pick(chosen)
      return true
    }
    return false
  }
  return { items, selected, onKey, pick }
}
