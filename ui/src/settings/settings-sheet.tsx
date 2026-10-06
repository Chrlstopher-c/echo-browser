// Responsabilite : feuille des reglages — apparence tenue par l'interface, puis les reglages du coeur par theme.

import type { ReactElement } from 'react'
import { EmptyState } from '../shared/design/empty-state'
import { IconSettings, IconWrench } from '../shared/design/icons'
import { ListRow } from '../shared/design/list-row'
import { SectionLabel } from '../shared/design/section-label'
import { SpacePicker } from '../spaces/space-picker'
import type { SpaceController } from '../spaces/use-space'
import type { ContainerActions } from '../tabs/use-containers'
import type { PermissionGrantView } from '../shared/contract'
import { ContainersSection } from './containers-section'
import { GrantsSection } from './grants-section'
import { SettingRow } from './setting-row'
import type { SettingsController } from './use-settings'

export interface SettingsSheetProps {
  settings: SettingsController
  space: SpaceController
  containers: ContainerActions
  grants: PermissionGrantView[]
  onForgetGrant: (origin: string, permission: string) => void
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
  const { settings, space, containers, grants, onForgetGrant, onDevTools } = props
  return (
    <div className="flex flex-col gap-3">
      <AppearanceSection space={space} />
      <ContainersSection actions={containers} />
      <GrantsSection grants={grants} onForget={onForgetGrant} />
      <CoreSections settings={settings} />
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
