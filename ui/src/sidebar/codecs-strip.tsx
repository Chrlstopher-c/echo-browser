// Responsabilite : proposer le decodeur video complet quand une page en a besoin (Twitch, MP4 en H.264/AAC), puis le
// redemarrage qui l'active. Meme place et meme matiere que la question d'autorisation d'un site.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { UiRequest, VideoCodecsView } from '../shared/contract'
import { QUICK } from '../shared/design/motion'
import { PushButton } from '../shared/design/push-button'

export interface CodecsStripProps {
  codecs: VideoCodecsView | null
  send: (request: UiRequest) => void
}

function Body({ codecs, send }: { codecs: VideoCodecsView; send: CodecsStripProps['send'] }): ReactElement | null {
  const later = (): void => send({ kind: 'dismissVideoCodecs', forever: false })
  switch (codecs.status) {
    case 'missing':
      return (
        <>
          <p className="text-ink">
            <span className="font-medium">{codecs.proposal}</span> diffuse une vidéo H.264/AAC qu’Echo ne lit pas
            d’emblée. Installer le décodeur ?
          </p>
          {codecs.error !== null && <p className="text-danger">{codecs.error}</p>}
          <div className="flex flex-wrap gap-1.5">
            <PushButton tone="guard" onClick={() => send({ kind: 'installVideoCodecs' })}>
              {codecs.error === null ? 'Installer' : 'Réessayer'}
            </PushButton>
            <PushButton onClick={later}>Plus tard</PushButton>
            <PushButton onClick={() => send({ kind: 'dismissVideoCodecs', forever: true })}>Jamais</PushButton>
          </div>
        </>
      )
    case 'downloading':
      return <p className="text-ink-muted">Téléchargement du décodeur…</p>
    case 'pendingRestart':
      return (
        <>
          <p className="text-ink">Décodeur installé. Redémarrer pour lire la vidéo ? Vos onglets sont gardés.</p>
          <div className="flex gap-1.5">
            <PushButton tone="guard" onClick={() => send({ kind: 'restartBrowser' })}>Redémarrer</PushButton>
            <PushButton onClick={later}>Plus tard</PushButton>
          </div>
        </>
      )
    default:
      return null
  }
}

export function CodecsStrip({ codecs, send }: CodecsStripProps): ReactElement {
  const shown = codecs !== null && codecs.proposal !== null &&
    ['missing', 'downloading', 'pendingRestart'].includes(codecs.status)
  return (
    <AnimatePresence>
      {shown && (
        <motion.div
          key="decodeur"
          role="alertdialog"
          aria-label="Décodeur vidéo"
          initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0 }}
          transition={QUICK}
          className="mb-1 flex flex-col gap-2 rounded-row bg-card px-2.5 py-2 text-[11.5px] leading-snug shadow-card"
        >
          <Body codecs={codecs} send={send} />
        </motion.div>
      )}
    </AnimatePresence>
  )
}
