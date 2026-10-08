// Responsabilite : panneau de l'historique — recherche, journees, retrait d'une visite, effacement.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { HistoryEntryView } from '../../shared/contract'
import { ConfirmStrip } from '../../shared/design/confirm-strip'
import { EmptyState } from '../../shared/design/empty-state'
import { IconClock, IconClose, IconTrash } from '../../shared/design/icons'
import { ListRow, RowAction } from '../../shared/design/list-row'
import { QUICK } from '../../shared/design/motion'
import { PushButton } from '../../shared/design/push-button'
import { SearchField } from '../../shared/design/search-field'
import { SectionLabel } from '../../shared/design/section-label'
import { SiteMark } from '../../shared/design/site-mark'
import { formatCount, formatDay, formatTime, fromCoreTime } from '../../shared/format'
import { readUrl } from '../../shared/url-shape'
import type { HistoryController, HistoryDay } from './use-history'

function HistoryRow({ entry, controller }: { entry: HistoryEntryView; controller: HistoryController }): ReactElement {
  const host = readUrl(entry.url).host
  return (
    <motion.div initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: 'auto' }}
      exit={{ opacity: 0, height: 0 }} transition={QUICK} className="overflow-hidden">
      <ListRow onClick={() => controller.open(entry.url)} title={entry.url}>
        <SiteMark url={entry.url} favicon={entry.favicon} size={14} />
        <span className="min-w-0 flex-1 truncate text-ink">{entry.title.length > 0 ? entry.title : host}</span>
        {entry.visits > 1 && (
          <span title={`${entry.visits} visites`}
            className="numerique shrink-0 rounded bg-ink/8 px-1 text-[10px] text-ink-faint">×{entry.visits}</span>
        )}
        <span className="numerique shrink-0 text-[10.5px] text-ink-faint group-hover:hidden">
          {formatTime(fromCoreTime(entry.visitedAt))}
        </span>
        <span className="hidden group-hover:block">
          <RowAction label="Retirer de l'historique" onClick={() => controller.remove(entry)} danger>
            <IconClose size={12} />
          </RowAction>
        </span>
      </ListRow>
    </motion.div>
  )
}

function DaySection({ day, controller }: { day: HistoryDay; controller: HistoryController }): ReactElement {
  return (
    <section>
      <SectionLabel>{formatDay(day.day)}</SectionLabel>
      <AnimatePresence initial={false}>
        {day.entries.map((entry) => (
          <HistoryRow key={`${entry.url}@${entry.visitedAt}`} entry={entry} controller={controller} />
        ))}
      </AnimatePresence>
    </section>
  )
}

function Footer({ controller }: { controller: HistoryController }): ReactElement {
  if (controller.confirmingClear) {
    return (
      <ConfirmStrip question="Effacer tout l’historique de navigation ?" confirmLabel="Tout effacer"
        onConfirm={controller.confirmClear} onCancel={controller.cancelClear} />
    )
  }
  const count = controller.total > controller.shown
    ? `${formatCount(controller.shown)} affichées sur ${formatCount(controller.total)}`
    : `${formatCount(controller.total)} ${controller.total > 1 ? 'entrées' : 'entrée'}`
  return (
    <div className="flex items-center justify-between gap-2 px-2">
      <p className="numerique truncate text-[10.5px] text-ink-faint">{count}</p>
      <PushButton tone="danger" onClick={controller.askClear} icon={<IconTrash size={11} />}>Effacer</PushButton>
    </div>
  )
}

export function HistoryPanel({ controller }: { controller: HistoryController }): ReactElement {
  const searching = controller.terms.trim().length > 0
  const empty = controller.days.length === 0
  return (
    <div className="flex flex-col gap-2">
      <div className="px-2">
        <SearchField value={controller.terms} placeholder="Rechercher dans l’historique"
          onChange={controller.setTerms} />
      </div>
      {empty ? (
        <EmptyState icon={<IconClock size={18} />} title={searching ? 'Aucun résultat' : 'Historique vide'}
          hint={searching ? 'Aucune page visitée ne correspond à cette recherche.'
            : 'Les pages visitées apparaîtront ici, jour par jour.'} />
      ) : (
        <div className="flex flex-col gap-1">
          {controller.days.map((day) => <DaySection key={day.day} day={day} controller={controller} />)}
        </div>
      )}
      {(controller.total > 0 || controller.confirmingClear) && <Footer controller={controller} />}
    </div>
  )
}
