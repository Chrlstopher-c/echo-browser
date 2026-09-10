// Responsabilite : feuille des extensions — installation, inventaire, renvoi au gestionnaire de Chromium.

import type { ReactElement } from 'react'
import { EmptyState } from '../shared/design/empty-state'
import { IconOpen, IconPuzzle } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'
import { SectionLabel } from '../shared/design/section-label'
import { ExtensionRow } from './extension-row'
import { InstallField } from './install-field'
import type { ExtensionsController } from './use-extensions'

const EMPTY_HINT = 'Collez l’adresse d’une extension du Chrome Web Store ci-dessus, ou parcourez le catalogue.'

function Inventory({ controller }: { controller: ExtensionsController }): ReactElement {
  if (controller.extensions.length === 0) {
    return <EmptyState icon={<IconPuzzle size={18} />} title="Aucune extension installée" hint={EMPTY_HINT} />
  }
  const manager = (
    <PushButton onClick={controller.openManager} icon={<IconOpen size={11} />}>Chromium</PushButton>
  )
  return (
    <section>
      <SectionLabel aside={manager}>Installées · {controller.extensions.length}</SectionLabel>
      <div className="divide-y divide-hairline border-t border-hairline">
        {controller.extensions.map((item) => (
          <ExtensionRow key={item.id} item={item} controller={controller} />
        ))}
      </div>
    </section>
  )
}

export function ExtensionsSheet({ controller }: { controller: ExtensionsController }): ReactElement {
  return (
    <div className="flex flex-col gap-1">
      <InstallField install={controller.install} onOpenStore={controller.openStore} />
      <Inventory controller={controller} />
    </div>
  )
}
