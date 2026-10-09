// Responsabilite : panneau des favoris — ajout de la page courante, liste reordonnable, retrait au survol.

import { AnimatePresence, Reorder, useDragControls } from 'framer-motion'
import { useRef, type ReactElement } from 'react'
import type { BookmarkView } from '../../shared/contract'
import { EmptyState } from '../../shared/design/empty-state'
import { IconClose, IconStar, IconStarFilled } from '../../shared/design/icons'
import { RowAction } from '../../shared/design/list-row'
import { QUICK } from '../../shared/design/motion'
import { PushButton } from '../../shared/design/push-button'
import { SiteMark } from '../../shared/design/site-mark'
import { readUrl } from '../../shared/url-shape'
import type { BookmarksController } from './use-bookmarks'

const ROW_MOTION = {
  initial: { opacity: 0, height: 0 },
  animate: { opacity: 1, height: 36 },
  exit: { opacity: 0, height: 0 },
  whileDrag: { scale: 1.02, boxShadow: 'var(--shadow-lift)', zIndex: 5 },
}

/** Un relachement de glisser passe aussi par le clic : il ne doit pas ouvrir la page. */
function useDragAwareClick(open: () => void): { onDragStart: () => void; onClick: () => void } {
  const dragged = useRef(false)
  return {
    onDragStart: () => {
      dragged.current = true
    },
    onClick: () => {
      if (dragged.current) dragged.current = false
      else open()
    },
  }
}

function BookmarkLabel({ item }: { item: BookmarkView }): ReactElement {
  const host = readUrl(item.url).host
  return (
    <span className="min-w-0 flex-1">
      <span className="block truncate text-[12.5px] text-ink">{item.title.length > 0 ? item.title : host}</span>
      <span className="numerique block truncate text-[10.5px] text-ink-faint">{host}</span>
    </span>
  )
}

function BookmarkRow({ item, controller }: { item: BookmarkView; controller: BookmarksController }): ReactElement {
  const controls = useDragControls()
  const click = useDragAwareClick(() => controller.open(item.url))
  return (
    <Reorder.Item
      value={item}
      dragListener={false}
      dragControls={controls}
      onDragStart={click.onDragStart}
      onDragEnd={() => controller.commitMove(item.url)}
      {...ROW_MOTION}
      transition={QUICK}
      title={item.url}
      onPointerDown={(event) => {
        if (event.button === 0) controls.start(event)
      }}
      onClick={click.onClick}
      className="group flex items-center gap-2.5 overflow-hidden rounded-row bg-shell px-2 transition-colors
        duration-100 hover:bg-hover"
    >
      <SiteMark url={item.url} favicon={item.favicon} size={16} />
      <BookmarkLabel item={item} />
      <RowAction label="Retirer des favoris" onClick={() => controller.remove(item.url)} danger>
        <IconClose size={12} />
      </RowAction>
    </Reorder.Item>
  )
}

function AddCurrent({ controller }: { controller: BookmarksController }): ReactElement {
  const label = controller.currentSaved ? 'Page enregistrée' : 'Enregistrer cette page'
  const hint = controller.currentSaved
    ? 'La page courante est dans vos favoris.'
    : controller.canAddCurrent
      ? 'La page courante peut être mise de côté.'
      : 'Sur une page web : Ctrl+D ou l’étoile de l’adresse.'
  return (
    <div className="flex items-center justify-between gap-2 px-2 pb-2">
      <p className="truncate text-[11px] text-ink-faint">{hint}</p>
      {(controller.canAddCurrent || controller.currentSaved) && <PushButton
        tone={controller.currentSaved ? 'neutral' : 'guard'}
        disabled={!controller.canAddCurrent}
        onClick={controller.addCurrent}
        icon={controller.currentSaved ? <IconStarFilled size={11} /> : <IconStar size={11} />}
      >
        {label}
      </PushButton>}
    </div>
  )
}

export function BookmarksPanel({ controller }: { controller: BookmarksController }): ReactElement {
  return (
    <div className="flex flex-col">
      <AddCurrent controller={controller} />
      {controller.order.length === 0 ? (
        <EmptyState icon={<IconStar size={18} />} title="Aucun favori"
          hint="Les pages mises de côté apparaîtront ici, dans l’ordre que vous leur donnez." />
      ) : (
        <Reorder.Group axis="y" values={controller.order} onReorder={controller.setOrder}
          className="flex flex-col gap-0.5">
          <AnimatePresence initial={false}>
            {controller.order.map((item) => <BookmarkRow key={item.url} item={item} controller={controller} />)}
          </AnimatePresence>
        </Reorder.Group>
      )}
    </div>
  )
}
