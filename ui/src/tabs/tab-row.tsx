// Responsabilite : un onglet de la liste — marque, titre tronque, son, fermeture au survol.
// L'onglet actif est une carte posee : le fond glisse d'une ligne a l'autre. Saisi, il se souleve.

import { motion, Reorder, useDragControls } from 'framer-motion'
import { useContext, useRef, type KeyboardEvent, type MouseEvent, type PointerEvent, type ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import { IconClose, IconVolumeOff } from '../shared/design/icons'
import { IconEyeOff } from '../shared/design/icons-page'
import { PANEL, QUICK } from '../shared/design/motion'
import { fallbackTitle } from '../shared/url-shape'
import { AudioBars, TabMark } from './tab-mark'
import { TabSend, useContainerName } from './container-names'
import { chosenContainer, containerColor } from './use-containers'

export interface DropPoint {
  x: number
  y: number
}

export interface TabRowProps {
  tab: TabView
  active: boolean
  compact: boolean
  onSelect: () => void
  onClose: () => void
  /** Survol prolonge d'un onglet endormi : on le reveille d'avance. */
  onWarm: () => void
  onContextMenu: (event: MouseEvent) => void
  /** Fin du glisser, avec le point de lacher (coordonnees de la fenetre). */
  onDragEnd: (point: DropPoint) => void
}

/** Delai de survol avant de reveiller un onglet endormi : assez court pour gagner du temps, assez long pour ignorer un passage. */
const WARM_DELAY_MS = 220

const ROW_MOTION = {
  transition: QUICK,
  initial: { opacity: 0, height: 0 },
  animate: { opacity: 1, height: 32 },
  exit: { opacity: 0, height: 0 },
  whileDrag: { scale: 1.02, boxShadow: 'var(--shadow-lift)', zIndex: 5 },
}

function CloseAction({ onClose }: { onClose: () => void }): ReactElement {
  return (
    <button
      type="button"
      aria-label="Fermer l'onglet"
      title="Fermer l'onglet"
      onPointerDown={(event) => event.stopPropagation()}
      onClick={(event) => {
        event.stopPropagation()
        onClose()
      }}
      className="grid size-5 shrink-0 place-items-center rounded-md text-ink-faint opacity-0
        transition-opacity duration-100 group-hover:opacity-100 hover:bg-ink/10 hover:text-ink
        focus-visible:opacity-100"
    >
      <IconClose size={12} />
    </button>
  )
}

/** Le conteneur d'un onglet, en clair : sa couleur et son nom (comme Firefox), pas un simple point. */
function ContainerChip({ context }: { context: string | null }): ReactElement | null {
  const id = chosenContainer(context)
  const named = useContainerName(id)
  if (id === 'prive') {
    return (
      <span data-container="Privé" title="Navigation privée : rien n’est gardé"
        className="flex shrink-0 items-center gap-1 rounded-full bg-ink px-1.5 text-[10px] leading-[15px] text-shell">
        <IconEyeOff size={9} /> Privé
      </span>
    )
  }
  const name = named
  if (id === null || name === null) return null
  return (
    <span data-container={name} title={`Conteneur : ${name}`}
      style={{ color: containerColor(id), borderColor: containerColor(id) }}
      className="max-w-[64px] shrink-0 truncate rounded-full border px-1.5 text-[10px] leading-[15px]">
      {name}
    </span>
  )
}

/** Le son d'un onglet : les barres quand il joue, un clic le coupe ; coupe, l'icone le dit et un clic le rend. */
function SoundButton({ tab }: { tab: TabView }): ReactElement {
  const send = useContext(TabSend)
  const label = tab.muted ? 'Remettre le son de l’onglet' : 'Couper le son de l’onglet'
  return (
    <button type="button" aria-label={label} title={label} aria-pressed={tab.muted}
      onPointerDown={(event) => event.stopPropagation()}
      onClick={(event) => {
        event.stopPropagation()
        send({ kind: 'setTabMuted', id: tab.id, muted: !tab.muted })
      }}
      className={`grid size-5 shrink-0 place-items-center rounded-md hover:bg-ink/10
        ${tab.muted ? 'text-ink-faint hover:text-ink' : 'text-guard'}`}>
      {tab.muted ? <IconVolumeOff size={12} /> : <AudioBars />}
    </button>
  )
}

function RowBody({ tab, title, onClose }: { tab: TabView; title: string; onClose: () => void }): ReactElement {
  return (
    <>
      <span className={`min-w-0 flex-1 truncate text-[12.5px] leading-none ${tab.asleep ? 'text-ink-faint' : ''}`}>
        {title}
      </span>
      <ContainerChip context={tab.container} />
      {((tab.audible && !tab.asleep) || tab.muted) && <SoundButton tab={tab} />}
      <span className="shrink-0">
        <CloseAction onClose={onClose} />
      </span>
    </>
  )
}

function ActiveBackdrop(): ReactElement {
  return (
    <motion.span layoutId="onglet-actif" transition={PANEL} className="absolute inset-0 -z-10 rounded-row shadow-card">
      <span className="absolute top-1/2 left-1.5 h-3.5 w-[3px] -translate-y-1/2 rounded-full bg-tint" />
    </motion.span>
  )
}

/** Clavier dans la liste : fleches pour passer d'un onglet a l'autre, Entree ou Espace pour l'ouvrir, Suppr pour le
 * fermer. Le focus suit l'ordre affiche (attribut `data-tab-row`). */
function onRowKey(event: KeyboardEvent<HTMLElement>, onSelect: () => void, onClose: () => void): void {
  const rows = [...document.querySelectorAll<HTMLElement>('[data-tab-row]')]
  const at = rows.indexOf(event.currentTarget)
  const focus = (index: number): void => rows[Math.max(0, Math.min(rows.length - 1, index))]?.focus()
  switch (event.key) {
    case 'ArrowDown': focus(at + 1); break
    case 'ArrowUp': focus(at - 1); break
    case 'Home': focus(0); break
    case 'End': focus(rows.length - 1); break
    case 'Enter': case ' ': onSelect(); break
    case 'Delete': case 'Backspace': onClose(); focus(at); break
    default: return
  }
  event.preventDefault()
}

/** Survol prolonge d'un onglet endormi : il se reveille d'avance. */
function useWarm(asleep: boolean, onWarm: () => void): { start: () => void; stop: () => void } {
  const timer = useRef(0)
  return {
    start: () => {
      if (asleep) timer.current = window.setTimeout(onWarm, WARM_DELAY_MS)
    },
    stop: () => window.clearTimeout(timer.current),
  }
}

export function TabRow(props: TabRowProps): ReactElement {
  const { tab, active, compact, onSelect, onClose, onWarm, onContextMenu, onDragEnd } = props
  const warm = useWarm(tab.asleep, onWarm)
  // Un titre qui n'est qu'une adresse interne d'Echo (titre pas encore recu) se lit mal : nom de repli.
  const title = tab.title.length > 0 && !tab.title.startsWith('echo://') ? tab.title : fallbackTitle(tab.url)
  const controls = useDragControls()
  const onPointerDown = (event: PointerEvent): void => {
    if (event.button !== 0) return
    onSelect()
    if (!compact) controls.start(event)
  }
  return (
    <Reorder.Item
      value={tab}
      dragListener={false}
      dragControls={controls}
      onDragEnd={(_, info) => onDragEnd({ x: info.point.x - window.scrollX, y: info.point.y - window.scrollY })}
      {...ROW_MOTION}
      onPointerDown={onPointerDown}
      onPointerEnter={warm.start}
      onPointerLeave={warm.stop}
      onContextMenu={onContextMenu}
      onKeyDown={(event: KeyboardEvent<HTMLElement>) => onRowKey(event, onSelect, onClose)}
      title={title}
      role="tab"
      aria-selected={active}
      aria-label={tab.asleep ? `${title} (endormi)` : title}
      tabIndex={active ? 0 : -1}
      data-tab-row=""
      className={`group relative isolate flex items-center gap-2.5 rounded-row outline-none
        focus-visible:ring-1 focus-visible:ring-guard/70
        ${compact ? 'justify-center px-0' : 'pr-1.5 pl-4'}
        ${active ? 'text-ink' : 'text-ink-muted hover:bg-hover hover:text-ink'}`}
    >
      {active && <ActiveBackdrop />}
      <TabMark tab={tab} size={16} />
      {!compact && <RowBody tab={tab} title={title} onClose={onClose} />}
    </Reorder.Item>
  )
}
