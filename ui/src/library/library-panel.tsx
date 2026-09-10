// Responsabilite : panneaux favoris, historique et téléchargements.

import type { ReactElement } from 'react'
import { EmptyState } from '../shared/design/empty-state'
import { IconClock, IconDownload, IconGlobe, IconStar } from '../shared/design/icons'
import { PanelRow, PanelShell } from '../shared/design/panel-shell'
import { readUrl } from '../shared/url-shape'
import {
  SECTION_TITLE,
  shortTime,
  type DownloadItem,
  type HistoryEntry,
  type LibraryContent,
  type LibrarySection,
} from './library-model'

const EMPTY_COPY: Record<LibrarySection, { title: string; hint: string }> = {
  bookmarks: {
    title: 'Aucun favori',
    hint: "Les pages mises de côté apparaîtront ici, dans l'ordre où vous les ajoutez.",
  },
  history: {
    title: 'Historique vide',
    hint: 'Les pages visitées pendant cette session apparaîtront ici.',
  },
  downloads: {
    title: 'Aucun téléchargement',
    hint: 'Les fichiers reçus apparaîtront ici, avec leur progression puis leur emplacement.',
  },
}

const EMPTY_ICON: Record<LibrarySection, ReactElement> = {
  bookmarks: <IconStar size={20} />,
  history: <IconClock size={20} />,
  downloads: <IconDownload size={20} />,
}

function LinkRow({ title, url, aside }: { title: string; url: string; aside?: string }): ReactElement {
  return (
    <PanelRow onClick={() => undefined}>
      <IconGlobe size={14} className="shrink-0 text-ink-faint" />
      <span className="min-w-0 flex-1 truncate text-ink">{title}</span>
      <span className="max-w-[38%] shrink-0 truncate text-[11.5px] text-ink-faint">{readUrl(url).host}</span>
      {aside !== undefined && <span className="numerique shrink-0 text-[11px] text-ink-faint">{aside}</span>}
    </PanelRow>
  )
}

function DownloadRow({ item }: { item: DownloadItem }): ReactElement {
  const tone = item.state === 'failed' ? 'text-danger' : item.state === 'done' ? 'text-ink' : 'text-guard'
  return (
    <PanelRow>
      <IconDownload size={14} className={`shrink-0 ${tone}`} />
      <span className="min-w-0 flex-1 truncate text-ink">{item.filename}</span>
      <span className="numerique shrink-0 text-[11px] text-ink-faint">
        {item.state === 'running' ? `${Math.round(item.progress * 100)} %` : ''}
      </span>
    </PanelRow>
  )
}

function historyRow(entry: HistoryEntry): ReactElement {
  return <LinkRow key={entry.id} title={entry.title} url={entry.url} aside={shortTime(entry.visitedAt)} />
}

export interface LibraryPanelProps {
  section: LibrarySection
  content: LibraryContent
}

export function LibraryPanel({ section, content }: LibraryPanelProps): ReactElement {
  const rows = buildRows(section, content)
  return (
    <PanelShell title={SECTION_TITLE[section]}>
      {rows.length === 0 ? (
        <EmptyState
          icon={EMPTY_ICON[section]}
          title={EMPTY_COPY[section].title}
          hint={EMPTY_COPY[section].hint}
        />
      ) : (
        <div className="py-1">{rows}</div>
      )}
    </PanelShell>
  )
}

function buildRows(section: LibrarySection, content: LibraryContent): ReactElement[] {
  if (section === 'bookmarks') {
    return content.bookmarks.map((item) => <LinkRow key={item.id} title={item.title} url={item.url} />)
  }
  if (section === 'history') return content.history.map(historyRow)
  return content.downloads.map((item) => <DownloadRow key={item.id} item={item} />)
}
