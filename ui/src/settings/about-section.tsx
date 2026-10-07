// Responsabilite : Reglages → A propos — version d'Echo, etat de la mise a jour, verification a la demande.

import type { ReactElement } from 'react'
import type { UpdateStatus, UpdateView } from '../shared/contract'
import { IconRefresh } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'
import { SectionLabel } from '../shared/design/section-label'

const STATUS_TEXT: Record<UpdateStatus, (u: UpdateView) => string> = {
  unavailable: () => 'Version de développement : pas de mise à jour automatique.',
  upToDate: () => 'Echo est à jour.',
  checking: () => 'Recherche d’une mise à jour…',
  available: (u) => `Echo ${u.latest ?? ''} est disponible.`,
  downloading: (u) => `Téléchargement d’Echo ${u.latest ?? ''}…`,
  ready: (u) => `Echo ${u.latest ?? ''} est prêt : il s’installe au prochain redémarrage.`,
  failed: (u) => `Mise à jour impossible : ${u.error ?? 'erreur inconnue'}.`,
}

export interface AboutSectionProps {
  update: UpdateView
  onCheck: () => void
}

export function AboutSection({ update, onCheck }: AboutSectionProps): ReactElement {
  const busy = update.status === 'checking' || update.status === 'downloading'
  return (
    <section>
      <SectionLabel aside={<span className="numerique text-[10.5px] text-ink-faint">{update.current}</span>}>
        À propos
      </SectionLabel>
      <div className="flex items-center gap-3 px-2 py-1">
        <p className="min-w-0 flex-1 text-[12.5px] leading-snug text-ink">{STATUS_TEXT[update.status](update)}</p>
        {update.status !== 'unavailable' && (
          <PushButton disabled={busy} onClick={onCheck} icon={<IconRefresh size={12} />}>
            {update.status === 'available' ? 'Installer' : 'Vérifier'}
          </PushButton>
        )}
      </div>
    </section>
  )
}
