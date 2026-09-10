// Responsabilite : panneau du bouclier — etat, interrupteurs, compteurs, listes de filtres.

import type { ReactElement } from 'react'
import type { ShieldView } from '../shared/contract'
import { PanelShell } from '../shared/design/panel-shell'
import { Toggle } from '../shared/design/toggle'
import { IconShield, IconShieldCheck } from '../shared/design/icons'
import { readUrl } from '../shared/url-shape'
import { SHIELD_LABEL, shieldModeOf } from './shield-reading'
import { ShieldStat } from './shield-stat'

export interface ShieldPanelProps {
  shield: ShieldView
  url: string
  filterListCount: number | null
  onToggleGlobal: (enabled: boolean) => void
  onToggleSite: () => void
  onRefreshLists: () => void
}

interface SwitchRowProps {
  title: string
  detail: string
  checked: boolean
  disabled?: boolean
  onChange: (next: boolean) => void
}

function SwitchRow({ title, detail, checked, disabled = false, onChange }: SwitchRowProps): ReactElement {
  return (
    <div className="flex items-center justify-between gap-4 px-3.5 py-2.5">
      <div className="min-w-0">
        <p className="truncate text-[12.5px] text-ink">{title}</p>
        <p className="truncate text-[11px] text-ink-faint">{detail}</p>
      </div>
      <Toggle checked={checked} onChange={onChange} label={title} disabled={disabled} />
    </div>
  )
}

function ShieldHeadline({ shield }: { shield: ShieldView }): ReactElement {
  const mode = shieldModeOf(shield)
  const tone = mode === 'active' ? 'text-guard' : mode === 'globalOff' ? 'text-warn' : 'text-ink-muted'
  const halo = mode === 'active' ? 'bg-guard/12' : 'bg-raised'
  return (
    <div className="flex items-center gap-3 px-3.5 pt-3.5 pb-3">
      <span className={`grid size-9 shrink-0 place-items-center rounded-full ${halo} ${tone}`}>
        {mode === 'active' ? <IconShieldCheck size={19} /> : <IconShield size={19} />}
      </span>
      <div className="min-w-0">
        <p className={`text-[13.5px] font-medium ${tone}`}>{SHIELD_LABEL[mode]}</p>
        <p className="truncate text-[11.5px] text-ink-faint">
          {mode === 'active' ? 'Traqueurs et publicités filtrés sur cette page.' : 'Aucun filtrage sur cette page.'}
        </p>
      </div>
    </div>
  )
}

export function ShieldPanel(props: ShieldPanelProps): ReactElement {
  const { shield, url, filterListCount, onToggleGlobal, onToggleSite, onRefreshLists } = props
  const host = readUrl(url).host
  return (
    <PanelShell title="Bouclier" footer={<FilterListFooter count={filterListCount} onRefresh={onRefreshLists} />}>
      <ShieldHeadline shield={shield} />
      <div className="grid grid-cols-2 gap-2 px-3.5 pb-3">
        <ShieldStat value={shield.blockedHere} caption="sur cette page" strong={shield.activeHere} />
        <ShieldStat value={shield.blockedTotal} caption="depuis le demarrage" />
      </div>
      <div className="divide-y divide-hairline border-t border-hairline">
        <SwitchRow
          title="Protection globale"
          detail="Filtrage appliqué à tous les sites"
          checked={shield.enabled}
          onChange={onToggleGlobal}
        />
        <SwitchRow
          title={host.length > 0 ? host : 'Ce site'}
          detail={shield.enabled ? 'Filtrage sur ce site' : 'Protection globale coupée'}
          checked={shield.activeHere}
          disabled={!shield.enabled}
          onChange={onToggleSite}
        />
      </div>
    </PanelShell>
  )
}

function FilterListFooter({ count, onRefresh }: { count: number | null; onRefresh: () => void }): ReactElement {
  return (
    <div className="flex items-center justify-between gap-3">
      <span>{count === null ? 'Listes de filtres chargées' : `${count} listes de filtres à jour`}</span>
      <button
        type="button"
        onClick={onRefresh}
        className="rounded border border-hairline px-2 py-[3px] text-[11px] text-ink-muted
          transition-colors duration-100 hover:border-edge hover:text-ink"
      >
        Rafraîchir
      </button>
    </div>
  )
}
