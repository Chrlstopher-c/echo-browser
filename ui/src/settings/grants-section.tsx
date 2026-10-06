// Responsabilite : les autorisations retenues par site (camera, micro, position…) et leur retrait.

import type { ReactElement } from 'react'
import type { PermissionGrantView } from '../shared/contract'
import { IconTrash } from '../shared/design/icons'
import { SectionLabel } from '../shared/design/section-label'

const KIND_LABELS: Record<string, string> = {
  camera: 'Caméra',
  microphone: 'Micro',
  geolocation: 'Position',
  notifications: 'Notifications',
  clipboard: 'Presse-papiers',
}

function hostOf(origin: string): string {
  try {
    return new URL(origin).host
  } catch {
    return origin
  }
}

export interface GrantsSectionProps {
  grants: PermissionGrantView[]
  onForget: (origin: string, permission: string) => void
}

export function GrantsSection({ grants, onForget }: GrantsSectionProps): ReactElement {
  return (
    <section>
      <SectionLabel>Autorisations des sites</SectionLabel>
      {grants.length === 0 ? (
        <p className="px-2 text-[11px] leading-snug text-ink-faint">
          Aucune décision retenue. Cochez « se souvenir » quand un site demande l’accès à la caméra ou au micro.
        </p>
      ) : (
        grants.map((grant) => (
          <div key={`${grant.origin}-${grant.kind}`} className="flex h-9 items-center gap-2 px-2">
            <span className="min-w-0 flex-1 truncate text-[12.5px] text-ink">{hostOf(grant.origin)}</span>
            <span className="text-[11px] text-ink-faint">
              {KIND_LABELS[grant.kind] ?? grant.kind} · {grant.allow ? 'autorisé' : 'refusé'}
            </span>
            <button
              type="button"
              aria-label={`Oublier ${grant.kind} pour ${hostOf(grant.origin)}`}
              title="Oublier cette décision"
              onClick={() => onForget(grant.origin, grant.kind)}
              className="grid size-6 place-items-center rounded-md text-ink-faint hover:bg-hover hover:text-danger"
            >
              <IconTrash size={13} />
            </button>
          </div>
        ))
      )}
    </section>
  )
}
