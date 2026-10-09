// Responsabilite : champ d'installation d'une extension — saisie, refus lisible, ouverture du catalogue.

import type { KeyboardEvent, ReactElement } from 'react'
import { IconOpen, IconPlus } from '../shared/design/icons'
import type { InstallField as InstallFieldState } from './use-extensions'

const PLACEHOLDER = 'ou collez l’adresse d’une extension'
const HINT = 'Sur la fiche d’une extension, « Ajouter à Echo ». Elle s’active au prochain démarrage d’Echo.'

export interface InstallFieldProps {
  install: InstallFieldState
  onOpenStore: () => void
}

function SourceInput({ install }: { install: InstallFieldState }): ReactElement {
  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>): void => {
    if (event.key === 'Enter') install.submit()
  }
  const frame = install.error === null ? 'focus-within:ring-1 focus-within:ring-guard/60' : 'ring-1 ring-danger/60'
  return (
    <div className={`flex h-8 items-center gap-1.5 rounded-row bg-field pr-1 pl-2.5 shadow-field ${frame}`}>
      <input
        value={install.value}
        spellCheck={false}
        autoComplete="off"
        aria-label="Source de l'extension"
        placeholder={PLACEHOLDER}
        onChange={(event) => install.change(event.target.value)}
        onKeyDown={onKeyDown}
        className="min-w-0 flex-1 bg-transparent text-[11.5px] text-ink outline-none select-text
          placeholder:text-ink-faint"
      />
      <button
        type="button"
        aria-label="Installer cette extension"
        title="Installer cette extension"
        onClick={install.submit}
        className="grid size-6 shrink-0 place-items-center rounded-row text-ink-muted transition-colors
          duration-100 hover:bg-hover hover:text-ink"
      >
        <IconPlus size={14} />
      </button>
    </div>
  )
}

export function InstallField({ install, onOpenStore }: InstallFieldProps): ReactElement {
  return (
    <div className="flex flex-col gap-1.5 px-2 pb-2">
      <button type="button" onClick={onOpenStore}
        className="flex h-9 items-center justify-center gap-2 rounded-row bg-card text-[12.5px] font-medium text-ink
          shadow-card transition-shadow duration-100 hover:text-guard active:shadow-pressed">
        <IconOpen size={13} /> Parcourir le catalogue d’extensions
      </button>
      <SourceInput install={install} />
      <p className={`px-0.5 text-[11px] leading-snug ${install.error === null ? 'text-ink-faint' : 'text-danger'}`}>
        {install.error ?? HINT}
      </p>
    </div>
  )
}
