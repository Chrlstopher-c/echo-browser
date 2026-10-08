// Responsabilite : feuille des reglages — apparence tenue par l'interface, puis les reglages du coeur par theme.

import type { ReactElement } from 'react'
import { EmptyState } from '../shared/design/empty-state'
import { IconSettings, IconWrench } from '../shared/design/icons'
import { ListRow } from '../shared/design/list-row'
import { SectionLabel } from '../shared/design/section-label'
import { SpacePicker } from '../spaces/space-picker'
import { readStoredScheme, type SpaceController } from '../spaces/use-space'
import type { ContainerActions } from '../tabs/use-containers'
import type { AccountView, PermissionGrantView, UiRequest, UpdateView, VideoCodecsView } from '../shared/contract'
import { AccountSection } from '../account/account-section'
import { AboutSection } from './about-section'
import { ContainersSection } from './containers-section'
import { ProfilesSection } from './profiles-section'
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
  send: (request: UiRequest) => void
  onDevTools: () => void
}

function AppearanceSection({ space }: { space: SpaceController }): ReactElement {
  return (
    <section>
      <SectionLabel aside={<span className="text-[10.5px] text-ink-faint">{space.space.name}</span>}>
        Espace
      </SectionLabel>
      <SpacePicker current={space.space.id} scheme={space.space.scheme} onSelect={space.select} />
      <p className="px-2 pt-1 text-[11px] leading-snug text-ink-faint">
        Teinte de la barre et du cadre autour de la page. Le choix est propre à cet ordinateur.
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
        <section key={section.group.id}>
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

export function SettingsSheet(props: SettingsSheetProps): ReactElement {
  const { settings, space, containers, profiles, grants, onForgetGrant, codecs, onCodecs, onDevTools } = props
  const { update, onCheckUpdate, account, send } = props
  return (
    <div className="flex flex-col gap-3">
      {account !== null && <AccountSection account={account} send={send} />}
      {space !== undefined && <AppearanceSection space={space} />}
      <ProfilesSection profiles={profiles} scheme={space?.space.scheme ?? readStoredScheme()} />
      <ContainersSection actions={containers} />
      <GrantsSection grants={grants} onForget={onForgetGrant} />
      {codecs !== null && <VideoSection view={codecs} onAction={onCodecs} />}
      <CoreSections settings={settings} />
      {update !== null && <AboutSection update={update} onCheck={onCheckUpdate} />}
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
