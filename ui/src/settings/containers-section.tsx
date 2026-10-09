// Responsabilite : conteneurs dans les reglages — nom et retrait ; l'ouverture se fait depuis la liste d'onglets.

import { useState, type ReactElement } from 'react'
import { IconTrash } from '../shared/design/icons'
import { SectionLabel } from '../shared/design/section-label'
import { containerColor, type Container, type ContainerActions } from '../tabs/use-containers'

function ContainerRow({ item, actions }: { item: Container; actions: ContainerActions }): ReactElement {
  const [name, setName] = useState(item.name)
  return (
    <div className="flex h-9 items-center gap-2.5 px-2">
      <span style={{ background: containerColor(item.id) }} className="block size-2.5 shrink-0 rounded-full" />
      <input
        value={name}
        aria-label="Nom du conteneur"
        onChange={(event) => setName(event.target.value)}
        onBlur={() => actions.rename(item.id, name)}
        onKeyDown={(event) => event.key === 'Enter' && event.currentTarget.blur()}
        className="h-7 min-w-0 flex-1 rounded-md bg-field px-2 text-[12.5px] text-ink shadow-field outline-none"
      />
      <button
        type="button"
        aria-label={`Retirer le conteneur ${item.name}`}
        title="Retirer le conteneur"
        onClick={() => actions.remove(item.id)}
        className="grid size-6 place-items-center rounded-md text-ink-faint hover:bg-hover hover:text-danger"
      >
        <IconTrash size={13} />
      </button>
    </div>
  )
}

export function ContainersSection({ actions }: { actions: ContainerActions }): ReactElement {
  return (
    <section>
      <SectionLabel>Conteneurs</SectionLabel>
      {actions.containers.length === 0 ? (
        <p className="px-2 text-[11.5px] leading-snug text-ink-faint">
          Un conteneur isole les cookies et les comptes d’un onglet. Clic droit dans la liste d’onglets pour en créer.
        </p>
      ) : (
        actions.containers.map((item) => <ContainerRow key={item.id} item={item} actions={actions} />)
      )}
    </section>
  )
}
