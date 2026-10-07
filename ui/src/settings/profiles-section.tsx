// Responsabilite : les profils dans les reglages — renommer chacun ; leur couleur est celle de leur pastille.

import { useState, type ReactElement } from 'react'
import { SectionLabel } from '../shared/design/section-label'
import { buildSpace, HUES, type Scheme, type SpaceId } from '../spaces/space-palette'
import type { ProfileNames } from '../spaces/use-profile-names'

function ProfileRow(props: { id: SpaceId; scheme: Scheme; profiles: ProfileNames }): ReactElement {
  const { id, scheme, profiles } = props
  const [name, setName] = useState(profiles.nameOf(id))
  return (
    <div className="flex h-9 items-center gap-2.5 px-2">
      <span style={{ background: buildSpace(id, scheme).tokens.tint }} className="block size-2.5 shrink-0 rounded-full" />
      <input
        value={name}
        aria-label="Nom du profil"
        onChange={(event) => setName(event.target.value)}
        onBlur={() => profiles.rename(id, name)}
        onKeyDown={(event) => event.key === 'Enter' && event.currentTarget.blur()}
        className="h-7 min-w-0 flex-1 rounded-md bg-field px-2 text-[12.5px] text-ink shadow-field outline-none"
      />
    </div>
  )
}

export function ProfilesSection({ profiles, scheme }: { profiles: ProfileNames; scheme: Scheme }): ReactElement {
  return (
    <section>
      <SectionLabel>Profils</SectionLabel>
      {HUES.map((hue) => <ProfileRow key={hue.id} id={hue.id} scheme={scheme} profiles={profiles} />)}
      <p className="px-2 pt-1 text-[11px] leading-snug text-ink-faint">
        Chaque profil a ses onglets et ses comptes ; le premier garde les connexions existantes.
      </p>
    </section>
  )
}
