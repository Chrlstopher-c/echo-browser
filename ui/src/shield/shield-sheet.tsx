// Responsabilite : feuille du bouclier — etat, compteurs, interrupteurs, listes de filtres.

import type { ReactElement } from 'react'
import type { ShieldView } from '../shared/contract'
import { IconShield, IconShieldCheck } from '../shared/design/icons'
import { Toggle } from '../shared/design/toggle'
import { readUrl } from '../shared/url-shape'
import { SHIELD_LABEL, shieldModeOf } from './shield-reading'
import { ShieldStat } from './shield-stat'

export interface ShieldSheetProps {
  shield: ShieldView
  url: string
  onToggleGlobal: (enabled: boolean) => void
  onToggleSite: () => void
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
    <div className="flex items-center justify-between gap-3 px-2 py-2">
      <div className="min-w-0">
        <p className="truncate text-[12.5px] text-ink">{title}</p>
        <p className="truncate text-[11px] text-ink-faint">{detail}</p>
      </div>
      <Toggle checked={checked} onChange={onChange} label={title} disabled={disabled} />
    </div>
  )
}

function Headline({ shield }: { shield: ShieldView }): ReactElement {
  const mode = shieldModeOf(shield)
  const tone = mode === 'active' ? 'text-guard' : mode === 'globalOff' ? 'text-warn' : 'text-ink-muted'
  return (
    <div className="flex items-center gap-2.5 px-2 pt-2 pb-3">
      <span className={`grid size-8 shrink-0 place-items-center rounded-full bg-card shadow-card ${tone}`}>
        {mode === 'active' ? <IconShieldCheck size={17} /> : <IconShield size={17} />}
      </span>
      <div className="min-w-0">
        <p className={`text-[12.5px] font-medium ${tone}`}>{SHIELD_LABEL[mode]}</p>
        <p className="truncate text-[11px] text-ink-faint">
          {mode === 'active' ? 'Traqueurs et publicités filtrés ici.' : 'Aucun filtrage sur cette page.'}
        </p>
      </div>
    </div>
  )
}

export function ShieldSheet({ shield, url, onToggleGlobal, onToggleSite }: ShieldSheetProps): ReactElement {
  const host = readUrl(url).host
  return (
    <>
      <Headline shield={shield} />
      <div className="grid grid-cols-2 gap-2 px-2 pb-3">
        <ShieldStat value={shield.blockedHere} caption="bloqués sur cette page" strong={shield.activeHere} />
        <ShieldStat value={shield.blockedTotal} caption="bloqués depuis le démarrage" />
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
    </>
  )
}

export interface FilterListFooterProps {
  count: number | null
  onRefresh: () => void
}

export function FilterListFooter({ count, onRefresh }: FilterListFooterProps): ReactElement {
  return (
    <div className="flex items-center justify-between gap-2 pb-2">
      <span className="truncate">
        {count === null ? 'Listes de filtres chargées' : `${count} listes de filtres à jour`}
      </span>
      <button
        type="button"
        onClick={onRefresh}
        className="shrink-0 rounded-md bg-card px-2 py-[3px] text-[11px] text-ink-muted shadow-card
          transition-colors duration-100 hover:text-ink"
      >
        Rafraîchir
      </button>
    </div>
  )
}
