// Responsabilite : les listes de filtres du bouclier — nom, nombre de regles, interrupteur, et le
// dernier rafraichissement avec son bouton.

import { motion } from 'framer-motion'
import { useState, type ReactElement } from 'react'
import type { FilterListView } from '../shared/contract'
import { formatCount, formatRelative, fromCoreTime } from '../shared/format'
import { IconRefresh } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'
import { Toggle } from '../shared/design/toggle'
import type { ShieldController } from './use-shield'

/** Ce que chaque liste fait, en langage courant ; la source technique passe en second. */
const PLAIN: Record<string, string> = {
  easylist: 'Publicités',
  easyprivacy: 'Traqueurs',
  'ubo-filters': 'Publicités et pistage (uBlock Origin)',
  'ubo-privacy': 'Vie privée (uBlock Origin)',
  'ubo-badware': 'Sites malveillants',
  'ubo-quick-fixes': 'Correctifs rapides',
  'ubo-unbreak': 'Réparations de sites',
  'liste-fr': 'Publicités des sites français',
  'peter-lowe': 'Régies et traqueurs connus',
  'easylist-cookie': 'Bandeaux de cookies',
  'fanboy-annoyances': 'Nuisances (fenêtres, surcouches)',
}

function rulesLabel(list: FilterListView): string {
  if (!list.enabled) return 'Désactivée'
  if (list.rules === null) return 'Chargement des règles…'
  return `${formatCount(list.rules)} règles`
}

function FilterListRow({ list, onToggle }: { list: FilterListView; onToggle: (next: boolean) => void }): ReactElement {
  return (
    <div className="flex h-10 items-center justify-between gap-3 px-2">
      <div className="min-w-0">
        <p className={`truncate text-[12.5px] ${list.enabled ? 'text-ink' : 'text-ink-muted'}`}>
          {PLAIN[list.id] ?? list.title}
        </p>
        <p className="truncate text-[10.5px] text-ink-faint">
          {rulesLabel(list)} · {list.title.split(' — ')[0]}
        </p>
      </div>
      <Toggle checked={list.enabled} onChange={onToggle} label={PLAIN[list.id] ?? list.title} />
    </div>
  )
}

function RefreshRow({ shield }: { shield: ShieldController }): ReactElement {
  const when = shield.refreshedAt === null ? 'jamais rafraîchies' : `rafraîchies ${formatRelative(fromCoreTime(shield.refreshedAt))}`
  return (
    <div className="flex items-center justify-between gap-2 px-2 pt-2 pb-1">
      <p className="truncate text-[11px] text-ink-faint">
        {shield.refreshing ? 'Rafraîchissement en cours…' : `Listes ${when}`}
      </p>
      <PushButton
        onClick={shield.refresh}
        disabled={shield.refreshing}
        icon={
          <motion.span
            animate={shield.refreshing ? { rotate: 360 } : { rotate: 0 }}
            transition={shield.refreshing ? { duration: 1, ease: 'linear', repeat: Infinity } : { duration: 0 }}
            className="grid place-items-center"
          >
            <IconRefresh size={11} />
          </motion.span>
        }
      >
        Rafraîchir
      </PushButton>
    </div>
  )
}

export function FilterLists({ shield }: { shield: ShieldController }): ReactElement {
  const [open, setOpen] = useState(false)
  const active = shield.lists.filter((list) => list.enabled).length
  const rules = shield.lists.reduce((sum, list) => sum + (list.enabled ? (list.rules ?? 0) : 0), 0)
  return (
    <section>
      <button type="button" aria-expanded={open} onClick={() => setOpen(!open)}
        className="flex w-full items-center justify-between gap-2 rounded-row px-2 py-1.5 text-left hover:bg-hover">
        <span className="text-[12px] text-ink-muted">Avancé : listes de filtres · {active}/{shield.lists.length}</span>
        <span className="numerique text-[10.5px] text-ink-faint">{formatCount(rules)} règles</span>
      </button>
      {open && <div className="divide-y divide-hairline">
        {shield.lists.map((list) => (
          <FilterListRow key={list.id} list={list} onToggle={(next) => shield.setListEnabled(list.id, next)} />
        ))}
        <RefreshRow shield={shield} />
      </div>}
    </section>
  )
}
