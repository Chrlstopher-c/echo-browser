// Responsabilite : les pages pleine largeur d'Echo — reglages et bibliotheque — dans un onglet, la ou la
// barre laterale manquait de place. Memes composants que la barre, autre mise en page.

import { useEffect, type ReactElement } from 'react'
import { LibrarySheet } from '../library/library-sheet'
import { useLibrary } from '../library/use-library'
import { SettingsSheet } from '../settings/settings-sheet'
import { videoRequest } from '../settings/video-section'
import { useSettings } from '../settings/use-settings'
import { AdminPage } from '../admin/admin-page'
import { HelpPage } from '../help/help-page'
import { useAdmin } from '../admin/use-admin'
import { IconHelp, IconLibrary, IconLock, IconPuzzle, IconSettings } from '../shared/design/icons'
import { ExtensionsSheet } from '../extensions/extensions-sheet'
import { useExtensions } from '../extensions/use-extensions'
import { useCore } from '../shared/use-core'
import { useProfileNames } from '../spaces/use-profile-names'
import { useContainers } from '../tabs/use-containers'
import { usePageRoute, type PageId } from './use-page-route'
import { usePageTheme } from './use-page-theme'
import { WelcomePage } from './welcome-page'

const PAGES: Array<{ id: PageId; label: string; icon: ReactElement }> = [
  { id: 'reglages', label: 'Réglages', icon: <IconSettings size={15} /> },
  { id: 'bibliotheque', label: 'Bibliothèque', icon: <IconLibrary size={15} /> },
  { id: 'extensions', label: 'Extensions', icon: <IconPuzzle size={15} /> },
  { id: 'aide', label: 'Aide', icon: <IconHelp size={15} /> },
  { id: 'admin', label: 'Administration', icon: <IconLock size={15} /> },
]

/** Pages montrees dans la navigation : l'administration seulement pour un compte administrateur. */
function visiblePages(admin: boolean): typeof PAGES {
  return PAGES.filter((p) => p.id !== 'admin' || admin)
}

function Nav({ page, go, admin }: { page: PageId; go: (next: PageId) => void; admin: boolean }): ReactElement {
  return (
    <nav className="flex shrink-0 flex-col gap-1 pt-2" aria-label="Pages">
      {visiblePages(admin).map((item) => (
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
  const extensions = useExtensions(send, state.extensions, state.restartPending, state.extensionPopupId)
  if (page === 'bibliotheque') return <LibrarySheet controller={library} />
  if (page === 'extensions') return <ExtensionsSheet controller={extensions} />
  if (page === 'admin' && state.account?.admin === true) return <Admin />
  if (page === 'aide') return <HelpPage send={send} />
  return (
    <SettingsSheet
      settings={settings}
      containers={containers}
      profiles={profiles}
      grants={state.grants}
      onForgetGrant={(origin, permission) => send({ kind: 'forgetPermission', origin, permission })}
        codecs={state.codecs}
        onCodecs={(action) => send(videoRequest(action))}
        update={state.update}
        onCheckUpdate={() => send({ kind: 'checkForUpdates' })}
        account={state.account}
        vault={state.vault}
        send={send}
      onDevTools={() => send({ kind: 'openDevTools', id: state.activeId ?? 0 })}
      importSources={state.importSources}
      {...(page === 'effacer' ? { focus: 'Effacer' } : {})}
    />
  )
}

function Admin(): ReactElement {
  const { state, send } = useCore()
  return <AdminPage admin={useAdmin(send, state.admin, state.adminAccount)} />
}

function Welcome(): ReactElement {
  const { state, send } = useCore()
  return (
    <WelcomePage account={state.account} settings={state.settings} importSources={state.importSources} send={send} />
  )
}

export function PagesApp(): ReactElement {
  usePageTheme()
  const [route, go] = usePageRoute()
  // « effacer » est un endroit des Reglages, pas une page a part.
  const page: PageId = route === 'effacer' ? 'reglages' : route
  const admin = useCore().state.account?.admin === true
  useEffect(() => {
    document.title = page === 'bienvenue' ? 'Bienvenue' : PAGES.find((p) => p.id === page)?.label ?? 'Echo'
  }, [page])
  if (page === 'bienvenue') return <Welcome />
  return (
    <div className="fond-espace h-screen overflow-y-auto bg-shell">
      <div className={`mx-auto flex gap-8 px-8 py-10 ${page === 'admin' ? 'max-w-6xl' : 'max-w-4xl'}`}>
        <Nav page={page} go={go} admin={admin} />
        <main className="min-w-0 flex-1 rounded-tile bg-card p-5 shadow-card">
          <h1 className="mb-4 text-[18px] font-semibold text-ink">{PAGES.find((p) => p.id === page)?.label}</h1>
          <Content page={route} />
        </main>
      </div>
    </div>
  )
}
