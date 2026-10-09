// Responsabilite : les profils dans les reglages — chacun est une identite (onglets, comptes, extensions a part) :
// renommer, changer sa teinte, en creer, reinitialiser (sessions effacees) ou supprimer. Le premier ne se supprime pas.

import { useState, type ReactElement } from 'react'
import { ConfirmStrip } from '../shared/design/confirm-strip'
import { IconPlus } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'
import { SectionLabel } from '../shared/design/section-label'
import { buildSpace, HUES, type Scheme } from '../spaces/space-palette'
import { DEFAULT_PROFILE, type ProfileEntry, type ProfileNames } from '../spaces/use-profile-names'

type Asking = 'reset' | 'remove' | null

const QUESTION: Record<'reset' | 'remove', string> = {
  reset: 'Fermer ses onglets et effacer ses sessions (déconnecté de partout) ; cache et stockage partent au prochain '
    + 'lancement.',
  remove: 'Supprimer ce profil, ses onglets, ses comptes et ses extensions propres. Sans retour.',
}

function ProfileRow({ entry, scheme, profiles }: { entry: ProfileEntry; scheme: Scheme; profiles: ProfileNames }):
  ReactElement {
  const [name, setName] = useState(entry.name)
  const [asking, setAsking] = useState<Asking>(null)
  const nextHue = HUES[(HUES.findIndex((h) => h.id === entry.hue) + 1) % HUES.length]?.id ?? entry.hue
  if (asking !== null) {
    return (
      <ConfirmStrip question={`${entry.name} : ${QUESTION[asking]}`}
        confirmLabel={asking === 'reset' ? 'Réinitialiser' : 'Supprimer'} onCancel={() => setAsking(null)}
        onConfirm={() => {
          if (asking === 'reset') profiles.reset(entry.id)
          else profiles.remove(entry.id)
          setAsking(null)
        }} />
    )
  }
  const main = entry.id === DEFAULT_PROFILE
  return (
    <div className="flex h-9 items-center gap-2 px-2">
      <button type="button" title="Changer de teinte" aria-label={`Teinte de ${entry.name}`}
        onClick={() => profiles.setHue(entry.id, nextHue)}
        className="grid size-5 shrink-0 place-items-center rounded-full shadow-card">
        <span style={{ background: buildSpace(entry.id, scheme, entry.hue).tokens.tint }}
          className="block size-2.5 rounded-full" />
      </button>
      <input value={name} aria-label="Nom du profil" onChange={(event) => setName(event.target.value)}
        onBlur={() => profiles.rename(entry.id, name)}
        onKeyDown={(event) => event.key === 'Enter' && event.currentTarget.blur()}
        className="h-7 min-w-0 flex-1 rounded-md bg-field px-2 text-[12.5px] text-ink shadow-field outline-none" />
      {main ? <span className="w-[178px] shrink-0 text-[10.5px] text-ink-faint">profil principal</span> : (
        <>
          <PushButton onClick={() => setAsking('reset')}>Réinitialiser</PushButton>
          <PushButton tone="danger" onClick={() => setAsking('remove')}>Supprimer</PushButton>
        </>
      )}
    </div>
  )
}

export function ProfilesSection({ profiles, scheme }: { profiles: ProfileNames; scheme: Scheme }): ReactElement {
  return (
    <section>
      <SectionLabel aside={
        <PushButton icon={<IconPlus size={12} />} onClick={() => profiles.create('')}>Nouveau profil</PushButton>
      }>Profils</SectionLabel>
      {profiles.list.map((entry) => <ProfileRow key={entry.id} entry={entry} scheme={scheme} profiles={profiles} />)}
      <p className="px-2 pt-1 text-[11.5px] leading-snug text-ink-faint">
        Chaque profil est une identité : ses onglets, ses comptes et ses extensions ne se mélangent pas aux autres. Le
        profil principal garde les connexions d'origine. On passe d'un profil à l'autre par les pastilles en bas de la
        barre.
      </p>
    </section>
  )
}
