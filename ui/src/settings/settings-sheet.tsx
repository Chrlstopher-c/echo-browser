// Responsabilite : feuille des reglages — espace courant, bibliotheque, outils.

import type { ReactElement, ReactNode } from 'react'
import { IconChevron, IconClock, IconStar } from '../shared/design/icons'
import { ListRow } from '../shared/design/list-row'
import { SectionLabel } from '../shared/design/section-label'
import type { SheetId } from '../sidebar/sheet'
import { SpacePicker } from '../spaces/space-picker'
import type { SpaceId } from '../spaces/space-palette'

export interface SettingsSheetProps {
  space: SpaceId
  spaceName: string
  onSelectSpace: (id: SpaceId) => void
  onOpen: (sheet: SheetId) => void
  onDevTools: () => void
}

function Entry({ icon, label, onClick }: { icon: ReactNode; label: string; onClick: () => void }): ReactElement {
  return (
    <ListRow onClick={onClick}>
      <span className="text-ink-muted">{icon}</span>
      <span className="flex-1 text-ink">{label}</span>
      <IconChevron size={13} className="text-ink-faint" />
    </ListRow>
  )
}

export function SettingsSheet(props: SettingsSheetProps): ReactElement {
  const { space, spaceName, onSelectSpace, onOpen, onDevTools } = props
  return (
    <div className="flex flex-col gap-3">
      <section>
        <SectionLabel>Espace · {spaceName}</SectionLabel>
        <SpacePicker current={space} onSelect={onSelectSpace} />
      </section>
      <section>
        <SectionLabel>Bibliothèque</SectionLabel>
        <Entry icon={<IconStar size={15} />} label="Favoris" onClick={() => onOpen('bookmarks')} />
        <Entry icon={<IconClock size={15} />} label="Historique" onClick={() => onOpen('history')} />
      </section>
      <section>
        <SectionLabel>Outils</SectionLabel>
        <ListRow onClick={onDevTools}>
          <span className="flex-1 text-ink-muted">Outils de développement</span>
        </ListRow>
      </section>
    </div>
  )
}
