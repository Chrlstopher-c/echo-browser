// Responsabilite : les pages pleine largeur d'Echo — reglages et bibliotheque — dans un onglet, la ou la
// barre laterale manquait de place. Memes composants que la barre, autre mise en page.

import { useEffect, type ReactElement } from 'react'
import { LibrarySheet } from '../library/library-sheet'
import { useLibrary } from '../library/use-library'
import { SettingsSheet } from '../settings/settings-sheet'
import { useSettings } from '../settings/use-settings'
import { IconLibrary, IconSettings } from '../shared/design/icons'
import { useCore } from '../shared/use-core'
import { useProfileNames } from '../spaces/use-profile-names'
import { useContainers } from '../tabs/use-containers'
import { usePageRoute, type PageId } from './use-page-route'
import { usePageTheme } from './use-page-theme'

const PAGES: Array<{ id: PageId; label: string; icon: ReactElement }> = [
  { id: 'reglages', label: 'Réglages', icon: <IconSettings size={15} /> },
  { id: 'bibliotheque', label: 'Bibliothèque', icon: <IconLibrary size={15} /> },
]

function Nav({ page, go }: { page: PageId; go: (next: PageId) => void }): ReactElement {
  return (
    <nav className="flex shrink-0 flex-col gap-1 pt-2" aria-label="Pages">
      {PAGES.map((item) => (
        <button
          key={item.id}
          type="button"
          onClick={() => go(item.id)}
          className={`flex h-9 items-center gap-2.5 rounded-row px-3 text-[13px] transition-shadow duration-150
            ${page === item.id ? 'text-ink shadow-card' : 'text-ink-muted hover:text-ink'}`}
        >
          {item.icon}
          {item.label}
        </button>
      ))}
    </nav>
  )
}

function Content({ page }: { page: PageId }): ReactElement {
  const core = useCore()
  const { state, send } = core
  const settings = useSettings(send, state.settings)
  const library = useLibrary(send, state)
  const containers = useContainers(send, state.settings)
  const profiles = useProfileNames(send, state.settings)
  if (page === 'bibliotheque') return <LibrarySheet controller={library} />
  return (
    <SettingsSheet
      settings={settings}
      containers={containers}
      profiles={profiles}
      grants={state.grants}
      onForgetGrant={(origin, permission) => send({ kind: 'forgetPermission', origin, permission })}
      onDevTools={() => send({ kind: 'openDevTools', id: state.activeId ?? 0 })}
    />
  )
}

export function PagesApp(): ReactElement {
  usePageTheme()
  const [page, go] = usePageRoute()
  useEffect(() => {
    document.title = PAGES.find((p) => p.id === page)?.label ?? 'Echo'
  }, [page])
  return (
    <div className="fond-espace min-h-screen bg-shell">
      <div className="mx-auto flex max-w-4xl gap-8 px-8 py-10">
        <Nav page={page} go={go} />
        <main className="min-w-0 flex-1 rounded-tile bg-card p-5 shadow-card">
          <h1 className="mb-4 text-[18px] font-semibold text-ink">{PAGES.find((p) => p.id === page)?.label}</h1>
          <Content page={page} />
        </main>
      </div>
    </div>
  )
}
