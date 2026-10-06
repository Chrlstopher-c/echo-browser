// Responsabilite : menu contextuel de la zone des onglets — ouvert au clic droit sur un onglet, un dossier
// ou le fond de la liste ; ferme au clic ailleurs ou sur Echap, position ramenee dans la fenetre.

import { useCallback, useEffect, useState, type MouseEvent } from 'react'
import type { TabId } from '../shared/contract'

export type MenuTarget =
  | { kind: 'tab'; id: TabId }
  | { kind: 'folder'; id: string }
  | { kind: 'area' }

export interface TabMenuState {
  target: MenuTarget
  x: number
  y: number
}

export interface TabMenuController {
  menu: TabMenuState | null
  openFor: (id: TabId) => (event: MouseEvent) => void
  openFolder: (id: string) => (event: MouseEvent) => void
  /** Clic droit sur le fond de la liste : seulement si aucun onglet ou dossier ne l'a pris avant. */
  openArea: (event: MouseEvent) => void
  close: () => void
}

const MENU_WIDTH = 224
const MENU_HEIGHT = 300
const MARGIN = 8

function clampToViewport(x: number, y: number): { x: number; y: number } {
  const maxX = window.innerWidth - MENU_WIDTH - MARGIN
  const maxY = window.innerHeight - MENU_HEIGHT - MARGIN
  return { x: Math.max(MARGIN, Math.min(x, maxX)), y: Math.max(MARGIN, Math.min(y, maxY)) }
}

/** Ferme le menu des qu'on clique hors de lui, qu'on presse Echap, ou que la fenetre perd le focus. */
function useDismiss(open: boolean, close: () => void): void {
  useEffect(() => {
    if (!open) return
    const onPointer = (event: PointerEvent): void => {
      const target = event.target
      if (target instanceof Element && target.closest('[data-tab-menu]') !== null) return
      close()
    }
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') {
        event.preventDefault()
        close()
      }
    }
    window.addEventListener('pointerdown', onPointer, true)
    window.addEventListener('keydown', onKey)
    window.addEventListener('blur', close)
    return () => {
      window.removeEventListener('pointerdown', onPointer, true)
      window.removeEventListener('keydown', onKey)
      window.removeEventListener('blur', close)
    }
  }, [open, close])
}

export function useTabMenu(): TabMenuController {
  const [menu, setMenu] = useState<TabMenuState | null>(null)
  const close = useCallback((): void => setMenu(null), [])
  const open = useCallback((target: MenuTarget) => (event: MouseEvent): void => {
    event.preventDefault()
    event.stopPropagation()
    setMenu({ target, ...clampToViewport(event.clientX, event.clientY) })
  }, [])
  const openFor = useCallback((id: TabId) => open({ kind: 'tab', id }), [open])
  const openFolder = useCallback((id: string) => open({ kind: 'folder', id }), [open])
  const openArea = useCallback((event: MouseEvent): void => open({ kind: 'area' })(event), [open])
  useDismiss(menu !== null, close)
  return { menu, openFor, openFolder, openArea, close }
}
