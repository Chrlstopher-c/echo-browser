// Responsabilite : feuille des reglages — apparence tenue par l'interface, puis les reglages du coeur par theme.

import { useEffect, useRef, useState, type ReactElement, type ReactNode } from 'react'
import { EmptyState } from '../shared/design/empty-state'
import { IconSettings, IconWrench } from '../shared/design/icons'
import { ListRow } from '../shared/design/list-row'
import { SectionLabel } from '../shared/design/section-label'
import { SpacePicker } from '../spaces/space-picker'
import {
  readSchemeChoice, readStoredScheme, writeSchemeChoice, type SchemeChoice, type SpaceController,
} from '../spaces/use-space'
import { Segmented, type Segment } from '../shared/design/segmented'
import type { ContainerActions } from '../tabs/use-containers'
import type {
  AccountView, ImportSourceView, PermissionGrantView, UiRequest, UpdateView, VaultKindView, VideoCodecsView,
} from '../shared/contract'
import { AccountSection } from '../account/account-section'
import { AboutSection } from './about-section'
import { ContainersSection } from './containers-section'
import { ProfilesSection } from './profiles-section'
import { FormsSection } from '../forms/forms-section'
import { ImportSection } from '../import/import-section'
import { ClearDataSection } from './clear-data-section'
import { useFormCards } from '../forms/form-cards'
import type { ProfileNames } from '../spaces/use-profile-names'
import { GrantsSection } from './grants-section'
import { VideoSection, type VideoAction } from './video-section'
import { SettingRow } from './setting-row'
import type { SettingsController } from './use-settings'

export interface SettingsSheetProps {
  settings: SettingsController
  /** Absent dans la page pleine largeur : la teinte se choisit dans la barre. */
  space?: SpaceController
  containers: ContainerActions
  profiles: ProfileNames
  grants: PermissionGrantView[]
  onForgetGrant: (origin: string, permission: string) => void
  codecs: VideoCodecsView | null
  onCodecs: (action: VideoAction) => void
  update: UpdateView | null
  onCheckUpdate: () => void
  account: AccountView | null
  vault: VaultKindView[] | null
  send: (request: UiRequest) => void
  onDevTools: () => void
  importSources: ImportSourceView[] | null
  /** Bloc a montrer d'emblee (Ctrl+Maj+Suppr : « Effacer »). */
  focus?: string
}

const SCHEMES: Array<Segment<SchemeChoice>> = [
  { id: 'light', label: 'Clair' },
  { id: 'dark', label: 'Sombre' },
  { id: 'system', label: 'Système' },
]

/** Theme : dans la barre il passe par l'espace, dans la page pleine largeur par le stockage partage. */
function ThemeSection({ space }: { space: SpaceController | undefined }): ReactElement {
  const [local, setLocal] = useState<SchemeChoice>(readSchemeChoice)
  const value = space?.schemeChoice ?? local
  const choose = (choice: SchemeChoice): void => {
    if (space !== undefined) space.setSchemeChoice(choice)
    else {
      writeSchemeChoice(choice)
      setLocal(choice)
    }
  }
  return (
    <section>
      <SectionLabel>Thème</SectionLabel>
      <div className="px-2">
        <Segmented name="scheme" segments={SCHEMES} value={value} onChange={choose} />
      </div>
      <p className="px-2 pt-1 text-[11px] leading-snug text-ink-faint">
        « Système » suit le thème clair ou sombre du bureau.
      </p>
    </section>
  )
}

function AppearanceSection({ space, profiles }: { space: SpaceController; profiles: ProfileNames }): ReactElement {
  return (
    <section>
      <SectionLabel aside={<span className="text-[10.5px] text-ink-faint">{space.space.name}</span>}>
        Espace
      </SectionLabel>
      <SpacePicker current={space.space.hue} scheme={space.space.scheme}
        onSelect={(hue) => profiles.setHue(space.space.id, hue)} />
      <p className="px-2 pt-1 text-[11px] leading-snug text-ink-faint">
        Teinte du profil courant : sa pastille, la barre et le cadre autour de la page.
      </p>
    </section>
  )
}

function CoreSections({ settings }: { settings: SettingsController }): ReactElement {
  if (settings.sections.length === 0) {
    return <EmptyState icon={<IconSettings size={18} />} title="Aucun réglage reçu"
      hint="Le cœur n’a pas encore livré ses réglages." />
  }
  return (
    <>
      {settings.sections.map((section) => (
        <section key={section.group.id} data-settings-part={section.group.title} className="scroll-mt-2">
          <SectionLabel>{section.group.title}</SectionLabel>
          <div className="divide-y divide-hairline">
            {section.entries.map((entry) => (
              <SettingRow key={entry.key} entry={entry} onChange={(value) => settings.update(entry.key, value)} />
            ))}
          </div>
        </section>
      ))}
    </>
  )
}

/** Un bloc repere par le sommaire. */
function Part({ title, children }: { title: string; children: ReactNode }): ReactElement {
  return <div data-settings-part={title} className="flex scroll-mt-2 flex-col gap-3">{children}</div>
}

/** Sommaire : un clic amene au bloc, les blocs absents (compte non charge…) ne sont pas proposes. */
function SettingsSummary({ version }: { version: string }): ReactElement {
  const [parts, setParts] = useState<string[]>([])
  const ref = useRef<HTMLElement>(null)
  useEffect(() => {
    const root = ref.current?.parentElement
    if (root === null || root === undefined) return
    setParts([...root.querySelectorAll<HTMLElement>('[data-settings-part]')].map((el) => el.dataset.settingsPart ?? ''))
  }, [version])
  const go = (title: string): void => {
    ref.current?.parentElement?.querySelector(`[data-settings-part="${title}"]`)?.scrollIntoView({ behavior: 'smooth' })
  }
  return (
    <nav ref={ref} aria-label="Sommaire des réglages" className="flex flex-wrap gap-1 px-1">
      {parts.map((title) => (
        <button key={title} type="button" onClick={() => go(title)}
          className="rounded-full bg-card px-2 py-0.5 text-[11px] text-ink-muted shadow-card hover:text-ink">
          {title}
        </button>
      ))}
    </nav>
  )
}

export function SettingsSheet(props: SettingsSheetProps): ReactElement {
  const { settings, space, containers, profiles, grants, onForgetGrant, codecs, onCodecs, onDevTools } = props
  const { update, onCheckUpdate, account, vault, send } = props
  const cards = useFormCards(send, settings.raw)
  useEffect(() => {
    if (props.focus === undefined) return
    const timer = window.setTimeout(() => {
      document.querySelector(`[data-settings-part="${props.focus}"]`)?.scrollIntoView({ block: 'start' })
    }, 120)
    return () => window.clearTimeout(timer)
  }, [props.focus])
  return (
    <div className="flex flex-col gap-3">
      <SettingsSummary version={`${settings.sections.length}${account !== null}${codecs !== null}${update !== null}`} />
      {account !== null && <Part title="Compte"><AccountSection account={account} vault={vault} send={send} /></Part>}
      <Part title="Apparence">
        <ThemeSection space={space} />
        {space !== undefined && <AppearanceSection space={space} profiles={profiles} />}
      </Part>
      <Part title="Profils">
        <ProfilesSection profiles={profiles} scheme={space?.space.scheme ?? readStoredScheme()} />
        <ContainersSection actions={containers} />
      </Part>
      <Part title="Formulaires"><FormsSection cards={cards} /></Part>
      <Part title="Effacer"><ClearDataSection send={send} /></Part>
      <Part title="Importer">
        <section>
          <SectionLabel>Importer d’un autre navigateur</SectionLabel>
          <div className="px-2"><ImportSection sources={props.importSources} send={send} /></div>
        </section>
      </Part>
      <Part title="Autorisations"><GrantsSection grants={grants} onForget={onForgetGrant} /></Part>
      {codecs !== null && <Part title="Vidéo"><VideoSection view={codecs} onAction={onCodecs} /></Part>}
      <CoreSections settings={settings} />
      {update !== null && <Part title="À propos"><AboutSection update={update} onCheck={onCheckUpdate} /></Part>}
      <section>
        <SectionLabel>Outils</SectionLabel>
        <ListRow onClick={onDevTools}>
          <IconWrench size={14} className="text-ink-muted" />
          <span className="flex-1 text-ink">Inspecter la page</span>
          <span className="numerique text-[10.5px] text-ink-faint">F12</span>
        </ListRow>
      </section>
    </div>
  )
}
