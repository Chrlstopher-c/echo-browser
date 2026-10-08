// Responsabilite : Reglages → Compte → donnees stockees — ce que le service garde, type par type, dechiffre sur cette
// machine ; chaque type se deplie sur ses elements.

import { AnimatePresence, motion } from 'framer-motion'
import { useState, type ReactElement } from 'react'
import type { VaultKindView } from '../shared/contract'
import { IconChevronDown } from '../shared/design/icons'
import { ListRow } from '../shared/design/list-row'
import { QUICK } from '../shared/design/motion'

const KIND_LABEL: Record<string, string> = {
  reglages: 'Réglages',
  favoris: 'Favoris',
  extensions: 'Extensions par profil',
  onglets: 'Onglets ouverts par machine',
  historique: 'Historique',
}
const SHOWN = 50

function size(bytes: number): string {
  if (bytes < 1024) return `${bytes} o`
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} Ko`
  return `${(bytes / 1024 / 1024).toFixed(1)} Mo`
}

function day(ms: number): string {
  return new Date(ms).toLocaleString('fr-FR', { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' })
}

function KindRow({ kind }: { kind: VaultKindView }): ReactElement {
  const [open, setOpen] = useState(false)
  const hidden = kind.count - Math.min(kind.lines.length, SHOWN)
  return (
    <div>
      <ListRow height={11} onClick={() => setOpen(!open)}>
        <div className="min-w-0 flex-1">
          <p className="truncate text-ink">{KIND_LABEL[kind.kind] ?? kind.kind}</p>
          <p className="numerique text-[10.5px] text-ink-faint">
            {kind.count} élément(s) · {size(kind.bytes)} chiffrés · {day(kind.updated)}
          </p>
        </div>
        <IconChevronDown size={13} className={`text-ink-faint transition-transform ${open ? 'rotate-180' : ''}`} />
      </ListRow>
      <AnimatePresence initial={false}>
        {open && (
          <motion.ul initial={{ height: 0, opacity: 0 }} animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }} transition={QUICK}
            className="ml-2 overflow-hidden border-l border-hairline pl-3">
            {kind.lines.slice(0, SHOWN).map((line, index) => (
              <li key={`${line.detail}-${index}`} className="py-0.5">
                <p className="truncate text-[11.5px] text-ink">{line.title === '' ? line.detail : line.title}</p>
                {line.title !== '' && <p className="truncate text-[10.5px] text-ink-faint">{line.detail}</p>}
              </li>
            ))}
            {hidden > 0 && <li className="py-1 text-[10.5px] text-ink-faint">et {hidden} autre(s)</li>}
          </motion.ul>
        )}
      </AnimatePresence>
    </div>
  )
}

export function VaultPanel({ kinds }: { kinds: VaultKindView[] }): ReactElement {
  if (kinds.length === 0) {
    return <p className="px-2 py-1 text-[11px] text-ink-faint">Rien n’est stocké sur le serveur pour ce compte.</p>
  }
  return (
    <div className="flex flex-col">
      {kinds.map((kind) => <KindRow key={kind.kind} kind={kind} />)}
      <p className="px-2 pt-1 text-[10.5px] leading-snug text-ink-faint">
        Le serveur ne voit que la version chiffrée : elle est déchiffrée ici, avec votre mot de passe.
      </p>
    </div>
  )
}
