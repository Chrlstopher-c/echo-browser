// Responsabilite : le menu contextuel d'une icone d'extension — ouvrir, regler, desactiver, retirer.
// Rendu par portail, comme celui des onglets, pour echapper au defilement de la barre.

import { AnimatePresence, motion } from 'framer-motion'
import { useCallback, useEffect, useState, type MouseEvent, type ReactElement, type ReactNode } from 'react'
import { createPortal } from 'react-dom'
import type { ExtensionView } from '../shared/contract'
import { IconClose, IconPuzzle, IconSettings } from '../shared/design/icons'
import { QUICK } from '../shared/design/motion'
import type { ExtensionsController } from './use-extensions'

const MENU_WIDTH = 208
const MENU_HEIGHT = 172
const MARGIN = 8

export interface ExtensionMenuState {
  id: string
  x: number
  y: number
}

export interface ExtensionMenuController {
  menu: ExtensionMenuState | null
  openFor: (id: string) => (event: MouseEvent) => void
  close: () => void
}

function clampToViewport(x: number, y: number): { x: number; y: number } {
  const maxX = window.innerWidth - MENU_WIDTH - MARGIN
  const maxY = window.innerHeight - MENU_HEIGHT - MARGIN
  return { x: Math.max(MARGIN, Math.min(x, maxX)), y: Math.max(MARGIN, Math.min(y, maxY)) }
}

/** Ferme le menu au clic ailleurs, a Echap, ou quand la fenetre perd le focus. */
function useDismiss(open: boolean, close: () => void): void {
  useEffect(() => {
    if (!open) return
    const onPointer = (event: PointerEvent): void => {
      const target = event.target
      if (target instanceof Element && target.closest('[data-extension-menu]') !== null) return
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

export function useExtensionMenu(): ExtensionMenuController {
  const [menu, setMenu] = useState<ExtensionMenuState | null>(null)
  const close = useCallback((): void => setMenu(null), [])
  const openFor = useCallback(
    (id: string) =>
      (event: MouseEvent): void => {
        event.preventDefault()
        event.stopPropagation()
        setMenu({ id, ...clampToViewport(event.clientX, event.clientY) })
      },
    [],
  )
  useDismiss(menu !== null, close)
  return { menu, openFor, close }
}

function MenuItem({ icon, label, onClick, danger = false, hint }: {
  icon: ReactNode; label: string; onClick: () => void; danger?: boolean; hint?: string
}): ReactElement {
  return (
    <button
      type="button"
      role="menuitem"
      onClick={onClick}
      title={hint}
      className={`flex h-7 w-full items-center gap-2.5 rounded-[6px] px-2 text-left text-[12px]
        transition-colors duration-100 hover:bg-hover
        ${danger ? 'text-ink-muted hover:text-danger' : 'text-ink'}`}
    >
      <span className="text-ink-muted">{icon}</span>
      {label}
    </button>
  )
}

interface MenuBodyProps {
  extension: ExtensionView
  controller: ExtensionsController
  onAskRemove: (id: string) => void
  close: () => void
}

function MenuBody({ extension, controller, onAskRemove, close }: MenuBodyProps): ReactElement {
  const run = (action: () => void) => (): void => {
    action()
    close()
  }
  return (
    <>
      <p className="truncate px-2 pt-1 pb-1.5 text-[11px] text-ink-faint">{extension.name}</p>
      {extension.options !== null && (
        <MenuItem
          icon={<IconSettings size={13} />}
          label="Réglages de l'extension"
          onClick={run(() => controller.openOptions(extension.id))}
        />
      )}
      <MenuItem
        icon={<IconPuzzle size={13} />}
        label={extension.enabled ? 'Désactiver' : 'Activer'}
        hint="Prend effet à la relance du navigateur"
        onClick={run(() => controller.setEnabled(extension.id, !extension.enabled))}
      />
      <div className="my-1 border-t border-hairline" />
      <MenuItem
        icon={<IconClose size={13} />}
        label="Retirer du navigateur"
        danger
        onClick={run(() => onAskRemove(extension.id))}
      />
    </>
  )
}

export interface ExtensionMenuProps {
  controller: ExtensionsController
  menu: ExtensionMenuController
  /** Le retrait se confirme dans le panneau : c'est lui qui porte la bande de confirmation. */
  onAskRemove: (id: string) => void
}

export function ExtensionMenu({ controller, menu, onAskRemove }: ExtensionMenuProps): ReactElement {
  const state = menu.menu
  const extension = state === null ? undefined : controller.extensions.find((item) => item.id === state.id)
  return createPortal(
    <AnimatePresence>
      {state !== null && extension !== undefined && (
        <motion.div
          key={state.id}
          data-extension-menu
          role="menu"
          initial={{ opacity: 0, scale: 0.96, y: -4 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.98 }}
          transition={QUICK}
          style={{ left: state.x, top: state.y, width: MENU_WIDTH }}
          className="fixed z-50 origin-top-left rounded-tile bg-card p-1 shadow-lift"
        >
          <MenuBody
            extension={extension}
            controller={controller}
            onAskRemove={onAskRemove}
            close={menu.close}
          />
        </motion.div>
      )}
    </AnimatePresence>,
    document.body,
  )
}
