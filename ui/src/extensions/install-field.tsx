// Responsabilite : champ d'installation d'une extension — saisie, refus lisible, attente du telechargement.

import type { KeyboardEvent, ReactElement } from 'react'
import { Spinner } from '../shared/design/spinner'
import { IconPlus } from '../shared/design/icons'
import type { InstallController } from './use-install'

const PLACEHOLDER = 'Adresse du Chrome Web Store, ou identifiant'

function SubmitButton({ install }: { install: InstallController }): ReactElement {
  if (install.busy) {
    return (
      <span className="grid size-6 shrink-0 place-items-center" aria-label="Installation en cours" role="status">
        <Spinner size={13} />
      </span>
    )
  }
  return (
    <button
      type="button"
      aria-label="Installer l'extension"
      title="Installer l'extension"
      onClick={install.submit}
      className="grid size-6 shrink-0 place-items-center rounded-row text-ink-muted transition-colors
        duration-100 hover:bg-hover hover:text-ink"
    >
      <IconPlus size={14} />
    </button>
  )
}

function hintOf(install: InstallController): string {
  if (install.error !== null) return install.error
  return install.busy ? 'Téléchargement depuis le catalogue…' : 'Collez une adresse ou un identifiant.'
}

export function InstallField({ install }: { install: InstallController }): ReactElement {
  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>): void => {
    if (event.key === 'Enter') install.submit()
  }
  const frame = install.error === null ? 'focus-within:ring-1 focus-within:ring-guard/60' : 'ring-1 ring-danger/60'
  return (
    <div className="px-2 pt-1 pb-2">
      <div className={`flex h-8 items-center gap-1.5 rounded-row bg-field pr-1 pl-2.5 shadow-card ${frame}`}>
        <input
          value={install.value}
          disabled={install.busy}
          spellCheck={false}
          autoComplete="off"
          aria-label="Source de l'extension"
          placeholder={PLACEHOLDER}
          onChange={(event) => install.change(event.target.value)}
          onKeyDown={onKeyDown}
          className="min-w-0 flex-1 bg-transparent text-[12px] text-ink outline-none select-text
            placeholder:text-ink-faint disabled:text-ink-faint"
        />
        <SubmitButton install={install} />
      </div>
      <p
        className={`px-0.5 pt-1 text-[11px] leading-snug
          ${install.error === null ? 'text-ink-faint' : 'text-danger'}`}
      >
        {hintOf(install)}
      </p>
    </div>
  )
}
