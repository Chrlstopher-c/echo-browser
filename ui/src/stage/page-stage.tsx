// Responsabilite : scene de developpement — la place de la page, dans un cadre flottant aux coins
// arrondis. Le coeur Rust peint ce cadre lui-meme ; ici il n'existe que pour juger la composition.

import type { ReactElement } from 'react'
import type { TabView } from '../shared/contract'
import { readUrl } from '../shared/url-shape'
import { STAGE_GUTTER } from '../sidebar/sidebar-geometry'

function FakeDocument({ tab }: { tab: TabView | null }): ReactElement {
  if (tab === null) {
    return <p className="text-[13px] text-page-ink/50">Aucun onglet ouvert.</p>
  }
  const shape = readUrl(tab.url)
  return (
    <div className="flex max-w-[52ch] flex-col items-center gap-2 text-center">
      <p className="text-[22px] font-medium tracking-[-0.02em] text-page-ink">{shape.host || tab.title}</p>
      <p className="numerique text-[12px] text-page-ink/55">{tab.url}</p>
      <p className="mt-4 text-[12px] text-page-ink/45">
        {tab.loading ? `Chargement… ${Math.round(tab.progress * 100)} %` : 'Ici, Chromium affiche la page.'}
      </p>
    </div>
  )
}

export function PageStage({ tab }: { tab: TabView | null }): ReactElement {
  return (
    <main
      style={{ padding: STAGE_GUTTER, paddingLeft: 0 }}
      className="flex min-w-0 flex-1 transition-colors duration-300"
    >
      <div className="grid flex-1 place-items-center overflow-hidden rounded-frame bg-page shadow-frame">
        <FakeDocument tab={tab} />
      </div>
    </main>
  )
}
