// Responsabilite : feuille des extensions — installation, inventaire, avertissement de relance.

import type { ReactElement } from 'react'
import { EmptyState } from '../shared/design/empty-state'
import { IconPuzzle } from '../shared/design/icons'
import { SectionLabel } from '../shared/design/section-label'
import { ExtensionRow } from './extension-row'
import { InstallField } from './install-field'
import type { ExtensionsController } from './use-extensions'

const EMPTY_HINT =
  "Collez l'adresse d'une extension du Chrome Web Store ci-dessus pour l'installer. " +
  'Elle sera chargée à la prochaine relance du navigateur.'

function Inventory({ controller }: { controller: ExtensionsController }): ReactElement {
  if (controller.extensions.length === 0) {
    return (
      <div className="pt-6">
        <EmptyState icon={<IconPuzzle size={20} />} title="Aucune extension installée" hint={EMPTY_HINT} />
      </div>
    )
  }
  return (
    <section>
      <SectionLabel>Installées · {controller.extensions.length}</SectionLabel>
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
    <div className="flex flex-col">
      <InstallField install={controller.install} />
      <Inventory controller={controller} />
    </div>
  )
}
