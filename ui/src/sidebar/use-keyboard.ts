// Responsabilite : raccourcis clavier quand la barre a le focus. Quand la page l'a, Chromium les
// recoit et le coeur relaie ce qui concerne l'interface (focusAddressRequested).

import { useEffect } from 'react'
import type { TabId } from '../shared/contract'
import type { SpaceController } from '../spaces/use-space'
import type { TabActions } from '../tabs/use-tab-actions'

interface Options {
  tabs: TabActions
  activeId: TabId | null
  space: SpaceController
  focusAddress: () => void
}

function isTyping(target: EventTarget | null): boolean {
  return target instanceof HTMLElement && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA')
}

export function useKeyboard({ tabs, activeId, space, focusAddress }: Options): void {
  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      const mod = event.ctrlKey || event.metaKey
      if (!mod) return
      const key = event.key.toLowerCase()
      if (key === 't') {
        event.preventDefault()
        tabs.newTab()
      } else if (key === 'w' && activeId !== null && !isTyping(event.target)) {
        event.preventDefault()
        tabs.close(activeId)
      } else if (key === 'r' && !isTyping(event.target)) {
        event.preventDefault()
        tabs.reloadActive()
      } else if (key === 'l') {
        event.preventDefault()
        focusAddress()
      } else if (event.altKey && (key === 'arrowdown' || key === 'arrowup')) {
        event.preventDefault()
        space.cycle(key === 'arrowdown' ? 1 : -1)
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [tabs, activeId, space, focusAddress])
}
