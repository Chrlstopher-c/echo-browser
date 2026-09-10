// Responsabilite : panneau des telechargements — progression en direct, ouverture, dossier, annulation, oubli.

import { AnimatePresence, motion } from 'framer-motion'
import type { ReactElement } from 'react'
import type { DownloadView } from '../../shared/contract'
import { EmptyState } from '../../shared/design/empty-state'
import { IconClose, IconDownload, IconFile, IconFolder, IconOpen } from '../../shared/design/icons'
import { RowAction } from '../../shared/design/list-row'
import { QUICK, TRACK } from '../../shared/design/motion'
import { PushButton } from '../../shared/design/push-button'
import { formatPercent } from '../../shared/format'
import { progressOf, stateLabel } from './download-reading'
import type { DownloadsController } from './use-downloads'

function ProgressBar({ item }: { item: DownloadView }): ReactElement {
  const progress = progressOf(item)
  return (
    <div className="relative mt-1 h-[3px] overflow-hidden rounded-full bg-hairline">
      {progress === null ? (
        <motion.span initial={{ x: '-100%' }} animate={{ x: '300%' }}
          transition={{ duration: 1.2, ease: 'easeInOut', repeat: Infinity }}
          className="absolute inset-y-0 w-1/3 rounded-full bg-guard" />
      ) : (
        <motion.span animate={{ scaleX: progress }} transition={TRACK}
          className="absolute inset-0 origin-left rounded-full bg-guard" />
      )}
    </div>
  )
}

function Actions({ item, controller }: { item: DownloadView; controller: DownloadsController }): ReactElement {
  if (item.state === 'running' || item.state === 'paused') {
    return (
      <RowAction label="Annuler le téléchargement" onClick={() => controller.cancel(item.id)} danger>
        <IconClose size={12} />
      </RowAction>
    )
  }
  return (
    <>
      {item.state === 'complete' && (
        <>
          <RowAction label="Ouvrir le fichier" onClick={() => controller.open(item.id)}>
            <IconOpen size={13} />
          </RowAction>
          <RowAction label="Afficher dans le dossier" onClick={() => controller.reveal(item.id)}>
            <IconFolder size={13} />
          </RowAction>
        </>
      )}
      <RowAction label="Retirer de la liste" onClick={() => controller.forget(item.id)} danger>
        <IconClose size={12} />
      </RowAction>
    </>
  )
}

function DownloadRow({ item, controller }: { item: DownloadView; controller: DownloadsController }): ReactElement {
  const running = item.state === 'running'
  const failed = item.state === 'failed' || item.state === 'cancelled'
  const progress = progressOf(item)
  return (
    <motion.div initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: 'auto' }}
      exit={{ opacity: 0, height: 0 }} transition={QUICK} className="overflow-hidden">
      <div title={item.url} className="group flex items-center gap-2.5 rounded-row px-2 py-2 hover:bg-hover">
        <span className={`grid size-7 shrink-0 place-items-center rounded-row bg-card shadow-card
          ${running ? 'text-guard' : failed ? 'text-ink-faint' : 'text-ink-muted'}`}>
          <IconFile size={15} />
        </span>
        <div className="min-w-0 flex-1">
          <p className={`truncate text-[12.5px] ${failed ? 'text-ink-muted line-through' : 'text-ink'}`}>
            {item.fileName}
          </p>
          <p className={`numerique truncate text-[10.5px]
            ${item.state === 'failed' ? 'text-danger' : 'text-ink-faint'}`}>
            {stateLabel(item)}
            {running && progress !== null && ` · ${formatPercent(progress)}`}
          </p>
          {running && <ProgressBar item={item} />}
        </div>
        <Actions item={item} controller={controller} />
      </div>
    </motion.div>
  )
}

export function DownloadsPanel({ controller }: { controller: DownloadsController }): ReactElement {
  const finished = controller.items.filter((item) => item.state !== 'running' && item.state !== 'paused').length
  if (controller.items.length === 0) {
    return <EmptyState icon={<IconDownload size={18} />} title="Aucun téléchargement"
      hint="Les fichiers reçus apparaîtront ici, avec leur progression en direct." />
  }
  return (
    <div className="flex flex-col gap-1">
      <div className="flex flex-col gap-0.5">
        <AnimatePresence initial={false}>
          {controller.items.map((item) => <DownloadRow key={item.id} item={item} controller={controller} />)}
        </AnimatePresence>
      </div>
      {finished > 0 && (
        <div className="flex justify-end px-2 pt-1">
          <PushButton onClick={controller.clearFinished}>Vider les terminés</PushButton>
        </div>
      )}
    </div>
  )
}
