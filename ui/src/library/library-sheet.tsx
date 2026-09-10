// Responsabilite : feuilles favoris, historique et telechargements.

import type { ReactElement } from 'react'
import { EmptyState } from '../shared/design/empty-state'
import { IconClock, IconDownload, IconStar } from '../shared/design/icons'
import { ListRow } from '../shared/design/list-row'
import { SiteMark } from '../shared/design/site-mark'
import {
  shortTime,
  type DownloadItem,
  type HistoryEntry,
  type LibraryContent,
  type LibrarySection,
} from './library-model'

const EMPTY_COPY: Record<LibrarySection, { title: string; hint: string }> = {
  bookmarks: { title: 'Aucun favori', hint: "Les pages mises de côté apparaîtront ici, dans l'ordre d'ajout." },
  history: { title: 'Historique vide', hint: 'Les pages visitées pendant cette session apparaîtront ici.' },
  downloads: { title: 'Aucun téléchargement', hint: 'Les fichiers reçus apparaîtront ici, avec leur progression.' },
}

const EMPTY_ICON: Record<LibrarySection, ReactElement> = {
  bookmarks: <IconStar size={20} />,
  history: <IconClock size={20} />,
  downloads: <IconDownload size={20} />,
}

function LinkRow({ title, url, aside }: { title: string; url: string; aside?: string }): ReactElement {
  return (
    <ListRow onClick={() => undefined}>
      <SiteMark url={url} favicon={null} size={14} />
      <span className="min-w-0 flex-1 truncate text-ink">{title}</span>
      {aside !== undefined && <span className="numerique shrink-0 text-[11px] text-ink-faint">{aside}</span>}
    </ListRow>
  )
}

function DownloadRow({ item }: { item: DownloadItem }): ReactElement {
  const tone = item.state === 'failed' ? 'text-danger' : item.state === 'done' ? 'text-ink' : 'text-guard'
  return (
    <ListRow>
      <IconDownload size={14} className={`shrink-0 ${tone}`} />
      <span className="min-w-0 flex-1 truncate text-ink">{item.filename}</span>
      <span className="numerique shrink-0 text-[11px] text-ink-faint">
        {item.state === 'running' ? `${Math.round(item.progress * 100)} %` : ''}
      </span>
    </ListRow>
  )
}

function historyRow(entry: HistoryEntry): ReactElement {
  return <LinkRow key={entry.id} title={entry.title} url={entry.url} aside={shortTime(entry.visitedAt)} />
}

function buildRows(section: LibrarySection, content: LibraryContent): ReactElement[] {
  if (section === 'bookmarks') {
    return content.bookmarks.map((item) => <LinkRow key={item.id} title={item.title} url={item.url} />)
  }
  if (section === 'history') return content.history.map(historyRow)
  return content.downloads.map((item) => <DownloadRow key={item.id} item={item} />)
}

export interface LibrarySheetProps {
  section: LibrarySection
  content: LibraryContent
}

export function LibrarySheet({ section, content }: LibrarySheetProps): ReactElement {
  const rows = buildRows(section, content)
  if (rows.length === 0) {
    return <EmptyState icon={EMPTY_ICON[section]} title={EMPTY_COPY[section].title} hint={EMPTY_COPY[section].hint} />
  }
  return <div className="flex flex-col gap-0.5">{rows}</div>
}
