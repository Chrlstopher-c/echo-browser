// Responsabilite : page Aide — ce qu'Echo sait faire, ou le trouver (avec un bouton qui y mene), et les raccourcis.

import { useState, type ReactElement } from 'react'
import type { UiRequest } from '../shared/contract'
import { PushButton } from '../shared/design/push-button'
import { SearchField } from '../shared/design/search-field'
import { SectionLabel } from '../shared/design/section-label'
import { HELP, SHORTCUTS, type HelpSection } from './help-content'

function Section({ section, send }: { section: HelpSection; send: (request: UiRequest) => void }): ReactElement {
  return (
    <section id={section.id}>
      <SectionLabel>{section.title}</SectionLabel>
      <p className="px-2 pb-1 text-[12px] leading-snug text-ink-muted">{section.intro}</p>
      <div className="divide-y divide-hairline">
        {section.topics.map((topic) => (
          <div key={topic.title} className="flex items-center gap-3 px-2 py-2">
            <div className="min-w-0 flex-1">
              <p className="text-[12.5px] text-ink">{topic.title}</p>
              <p className="text-[11.5px] leading-snug text-ink-faint">{topic.how}</p>
            </div>
            {topic.action !== undefined && (
              <PushButton onClick={() => topic.action !== undefined && send(topic.action.request)}>
                {topic.action.label}
              </PushButton>
            )}
          </div>
        ))}
      </div>
    </section>
  )
}

function Shortcuts({ query }: { query: string }): ReactElement {
  const shown = SHORTCUTS.filter(([keys, what]) => fold(`${keys} ${what}`).includes(fold(query.trim())))
  return (
    <section id="raccourcis">
      <SectionLabel>Raccourcis clavier</SectionLabel>
      <div className="grid grid-cols-1 gap-x-6 px-2 md:grid-cols-2">
        {shown.map(([keys, what]) => (
          <div key={keys} className="flex items-center justify-between gap-3 border-b border-hairline py-1.5">
            <span className="text-[12px] text-ink-muted">{what}</span>
            <kbd className="numerique shrink-0 rounded-md bg-field px-1.5 py-0.5 text-[11px] text-ink shadow-field">
              {keys}
            </kbd>
          </div>
        ))}
      </div>
    </section>
  )
}

/** Sans accents ni casse : « securite » trouve « Sécurité ». */
function fold(text: string): string {
  return text.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase()
}

/** Les sections reduites aux sujets dont le titre ou le geste contient tous les mots cherches. */
function filterHelp(query: string): HelpSection[] {
  const words = fold(query).split(/\s+/).filter(Boolean)
  if (words.length === 0) return HELP
  return HELP.map((section) => ({
    ...section,
    topics: section.topics.filter((t) => words.every((w) => fold(`${t.title} ${t.how} ${section.title}`).includes(w))),
  })).filter((section) => section.topics.length > 0)
}

export function HelpPage({ send }: { send: (request: UiRequest) => void }): ReactElement {
  const [query, setQuery] = useState('')
  const sections = filterHelp(query)
  const shortcuts = query.trim().length === 0
    || SHORTCUTS.some(([keys, what]) => fold(`${keys} ${what}`).includes(fold(query.trim())))
  return (
    <div className="flex flex-col gap-4">
      <p className="px-2 text-[12.5px] leading-snug text-ink-muted">
        Echo garde vos sessions, bloque les traqueurs et vous montre ce que font les sites. Voici où trouver chaque
        chose. F1 rouvre cette page à tout moment.
      </p>
      <div className="px-2">
        <SearchField value={query} placeholder="Chercher dans l’Aide" onChange={setQuery} autoFocus />
      </div>
      {sections.map((section) => <Section key={section.id} section={section} send={send} />)}
      {sections.length === 0 && !shortcuts && (
        <p className="px-2 text-[12px] text-ink-faint">Rien ne correspond à « {query} ».</p>
      )}
      {shortcuts && <Shortcuts query={query} />}
    </div>
  )
}
