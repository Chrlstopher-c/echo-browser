// Responsabilite : la rangee d'icones des extensions, et l'ouverture de leur fenetre.

import { useCallback, useState, type MouseEvent, type ReactElement } from 'react'
import type { AnchorRect, ExtensionView } from '../shared/contract'
import { IconPuzzle } from '../shared/design/icons'

export interface ExtensionBarProps {
  extensions: ExtensionView[]
  /** Extension dont la fenetre est ouverte, s'il y en a une. */
  openId: string | null
  onOpen: (id: string, anchor: AnchorRect) => void
  /** Menu de gestion, ouvert au clic droit sur une icone. */
  onMenu: (id: string) => (event: MouseEvent) => void
}

/** Une extension n'a sa place dans la barre que si elle est active et sait s'ouvrir. */
export function barExtensions(extensions: ExtensionView[]): ExtensionView[] {
  return extensions.filter((item) => item.enabled && item.popup !== null && !item.pending)
}

function anchorOf(element: HTMLElement): AnchorRect {
  const box = element.getBoundingClientRect()
  return {
    x: Math.round(box.left),
    y: Math.round(box.top),
    width: Math.round(box.width),
    height: Math.round(box.height),
  }
}

interface ExtensionIconProps {
  extension: ExtensionView
  open: boolean
  onOpen: (id: string, anchor: AnchorRect) => void
  onMenu: (event: MouseEvent) => void
}

function ExtensionIcon({ extension, open, onOpen, onMenu }: ExtensionIconProps): ReactElement {
  // Un paquet peut ne pas fournir d'icone lisible : le bouton garde alors la piece de
  // puzzle plutot que de devenir un carre vide.
  const [failed, setFailed] = useState(false)
  const click = useCallback(
    (event: MouseEvent<HTMLButtonElement>): void => onOpen(extension.id, anchorOf(event.currentTarget)),
    [extension.id, onOpen],
  )
  return (
    <button
      type="button"
      title={extension.name}
      aria-label={extension.name}
      aria-pressed={open}
      onClick={click}
      onContextMenu={onMenu}
      className={`relative flex size-7 shrink-0 items-center justify-center rounded-row text-ink-muted
        transition-colors duration-100 hover:bg-hover hover:text-ink ${open ? 'bg-card text-ink shadow-card' : ''}`}
    >
      {extension.icon === null || failed ? (
        <IconPuzzle size={15} />
      ) : (
        <img
          src={extension.icon}
          alt=""
          className="size-4 rounded-[3px] object-contain"
          onError={() => setFailed(true)}
        />
      )}
    </button>
  )
}

export function ExtensionBar({ extensions, openId, onOpen, onMenu }: ExtensionBarProps): ReactElement | null {
  const shown = barExtensions(extensions)
  if (shown.length === 0) return null
  return (
    <div className="flex items-center gap-0.5">
      {shown.map((extension) => (
        <ExtensionIcon
          key={extension.id}
          extension={extension}
          open={openId === extension.id}
          onOpen={onOpen}
          onMenu={onMenu(extension.id)}
        />
      ))}
    </div>
  )
}
