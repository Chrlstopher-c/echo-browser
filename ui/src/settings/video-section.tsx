// Responsabilite : decodeur video complet (H.264/AAC) — etat, installation a la demande depuis un tiers, retrait.

import type { ReactElement } from 'react'
import type { UiRequest, VideoCodecsStatus, VideoCodecsView } from '../shared/contract'
import { IconDownload, IconRefresh, IconTrash } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'
import { SectionLabel } from '../shared/design/section-label'

export type VideoAction = 'install' | 'remove' | 'restart'

export function videoRequest(action: VideoAction): UiRequest {
  if (action === 'install') return { kind: 'installVideoCodecs' }
  if (action === 'remove') return { kind: 'removeVideoCodecs' }
  return { kind: 'restartBrowser' }
}

const STATUS_TEXT: Record<VideoCodecsStatus, string> = {
  builtIn: 'Décodeur complet inclus : H.264, AAC et HEVC sont lus.',
  unavailable: 'Le moteur intègre son propre décodeur : rien à installer.',
  missing: 'VP9, AV1 et Opus sont lus. H.264 et AAC (Twitch, certains MP4) demandent le décodeur complet.',
  downloading: 'Téléchargement du décodeur complet…',
  pendingRestart: 'Décodeur complet installé : actif après redémarrage.',
  active: 'Décodeur complet actif : H.264 et AAC sont lus.',
  pendingRemoval: 'Décodeur complet retiré : le décodeur libre reprend après redémarrage.',
}

function Action({ view, onAction }: VideoSectionProps): ReactElement | null {
  switch (view.status) {
    case 'missing':
    case 'downloading':
      return (
        <PushButton tone="guard" disabled={view.status === 'downloading'} onClick={() => onAction('install')}
          icon={<IconDownload size={12} />}>
          Installer
        </PushButton>
      )
    case 'pendingRestart':
    case 'pendingRemoval':
      return (
        <PushButton tone="warn" onClick={() => onAction('restart')} icon={<IconRefresh size={12} />}>
          Redémarrer
        </PushButton>
      )
    case 'active':
      return (
        <PushButton tone="danger" onClick={() => onAction('remove')} icon={<IconTrash size={12} />}>
          Retirer
        </PushButton>
      )
    default:
      return null
  }
}

export interface VideoSectionProps {
  view: VideoCodecsView
  onAction: (action: VideoAction) => void
}

export function VideoSection({ view, onAction }: VideoSectionProps): ReactElement {
  const thirdParty = view.status === 'missing' || view.status === 'downloading' || view.status === 'active'
  return (
    <section>
      <SectionLabel>Vidéo</SectionLabel>
      <div className="flex items-center gap-3 px-2 py-1">
        <p className="min-w-0 flex-1 text-[12.5px] leading-snug text-ink">{STATUS_TEXT[view.status]}</p>
        <Action view={view} onAction={onAction} />
      </div>
      {thirdParty && (
        <p className="px-2 text-[11px] leading-snug text-ink-faint">
          H.264 et AAC sont brevetés : Echo ne les distribue pas. Le décodeur vient de {view.source}, vérifié par
          empreinte avant d’être gardé.
        </p>
      )}
      {view.error !== null && <p className="px-2 pt-1 text-[11px] leading-snug text-danger">{view.error}</p>}
    </section>
  )
}
