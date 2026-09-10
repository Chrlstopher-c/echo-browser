// Responsabilite : feuille du bouclier — etat, compteurs, interrupteurs global et par site, listes.

import type { ReactElement } from 'react'
import type { ShieldView } from '../shared/contract'
import { IconShield, IconShieldCheck } from '../shared/design/icons'
import { Toggle } from '../shared/design/toggle'
import { readUrl } from '../shared/url-shape'
import { FilterLists } from './filter-lists'
import { SHIELD_DETAIL, SHIELD_LABEL, shieldModeOf } from './shield-reading'
import { ShieldStat } from './shield-stat'
import type { ShieldController } from './use-shield'

export interface ShieldSheetProps {
  view: ShieldView
  url: string
  shield: ShieldController
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
    <div className="flex h-11 items-center justify-between gap-3 px-2">
      <div className="min-w-0">
        <p className="truncate text-[12.5px] text-ink">{title}</p>
        <p className="truncate text-[11px] text-ink-faint">{detail}</p>
      </div>
      <Toggle checked={checked} onChange={onChange} label={title} disabled={disabled} />
    </div>
  )
}

function Headline({ view }: { view: ShieldView }): ReactElement {
  const mode = shieldModeOf(view)
  const tone = mode === 'active' ? 'text-guard' : mode === 'globalOff' ? 'text-warn' : 'text-ink-muted'
  return (
    <div className="flex items-center gap-2.5 px-2 pt-1 pb-3">
      <span className={`grid size-8 shrink-0 place-items-center rounded-full bg-card shadow-card ${tone}`}>
        {mode === 'active' ? <IconShieldCheck size={17} /> : <IconShield size={17} />}
      </span>
      <div className="min-w-0">
        <p className={`text-[12.5px] font-medium ${tone}`}>{SHIELD_LABEL[mode]}</p>
        <p className="truncate text-[11px] text-ink-faint">{SHIELD_DETAIL[mode]}</p>
      </div>
    </div>
  )
}

export function ShieldSheet({ view, url, shield }: ShieldSheetProps): ReactElement {
  const host = readUrl(url).host
  return (
    <div className="flex flex-col gap-2">
      <Headline view={view} />
      <div className="grid grid-cols-2 gap-2 px-2 pb-1">
        <ShieldStat value={view.blockedHere} caption="bloqués sur cette page" strong={view.activeHere} />
        <ShieldStat value={view.blockedTotal} caption="bloqués depuis le démarrage" />
      </div>
      <div className="divide-y divide-hairline border-y border-hairline">
        <SwitchRow
          title="Protection globale"
          detail="Filtrage appliqué à tous les sites"
          checked={view.enabled}
          onChange={shield.setEnabled}
        />
        <SwitchRow
          title={host.length > 0 ? host : 'Ce site'}
          detail={view.enabled ? 'Filtrage sur ce site' : 'Protection globale coupée'}
          checked={view.activeHere}
          disabled={!view.enabled}
          onChange={shield.toggleSite}
        />
      </div>
      <FilterLists shield={shield} />
    </div>
  )
}
