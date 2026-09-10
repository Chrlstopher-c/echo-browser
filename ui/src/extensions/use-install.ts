// Responsabilite : champ d'installation — saisie, validation locale, attente du telechargement.

import { useCallback, useEffect, useReducer, useRef, type Dispatch, type RefObject } from 'react'
import type { UiRequest } from '../shared/contract'
import { readExtensionSource } from './extension-source'

/** Garde-fou : au-dela, on rend la main plutot que de laisser l'attente tourner indefiniment. */
const INSTALL_TIMEOUT_MS = 60_000

const TIMEOUT_MESSAGE = "L'installation n'a rien renvoyé. Réessayez, ou vérifiez l'identifiant."

const ALREADY_HERE = 'Cette extension est déjà installée.'

interface FieldState {
  value: string
  error: string | null
  busy: boolean
}

type FieldAction =
  | { kind: 'change'; value: string }
  | { kind: 'reject'; reason: string }
  | { kind: 'begin' }
  | { kind: 'settle' }
  | { kind: 'timeout' }

const EMPTY_FIELD: FieldState = { value: '', error: null, busy: false }

function reduceField(state: FieldState, action: FieldAction): FieldState {
  switch (action.kind) {
    case 'change':
      return { ...state, value: action.value, error: null }
    case 'reject':
      return { ...state, error: action.reason }
    case 'begin':
      return { ...state, error: null, busy: true }
    case 'settle':
      return EMPTY_FIELD
    case 'timeout':
      return { ...state, busy: false, error: TIMEOUT_MESSAGE }
  }
}

export interface InstallController extends FieldState {
  change: (next: string) => void
  submit: () => void
}

interface Options {
  send: (request: UiRequest) => void
  /** Empreinte de l'inventaire : sa mutation signe la fin de l'installation. */
  signature: string
  /** Identifiants deja presents : on refuse le doublon sans passer par le coeur. */
  installed: string[]
}

interface Watch {
  awaited: RefObject<string | null>
  busy: boolean
  signature: string
  dispatch: Dispatch<FieldAction>
}

/** Fin de l'attente : soit l'inventaire a bouge, soit le garde-fou de temps a parle. */
function useInstallWatch({ awaited, busy, signature, dispatch }: Watch): void {
  useEffect(() => {
    if (awaited.current === null || awaited.current === signature) return
    awaited.current = null
    dispatch({ kind: 'settle' })
  }, [awaited, dispatch, signature])

  useEffect(() => {
    if (!busy) return
    const timer = setTimeout(() => {
      awaited.current = null
      dispatch({ kind: 'timeout' })
    }, INSTALL_TIMEOUT_MS)
    return () => clearTimeout(timer)
  }, [awaited, busy, dispatch])
}

export function useInstall({ send, signature, installed }: Options): InstallController {
  const [field, dispatch] = useReducer(reduceField, EMPTY_FIELD)
  const awaited = useRef<string | null>(null)
  useInstallWatch({ awaited, busy: field.busy, signature, dispatch })

  const change = useCallback((next: string): void => dispatch({ kind: 'change', value: next }), [])

  const submit = useCallback((): void => {
    if (field.busy) return
    const check = readExtensionSource(field.value)
    if (!check.ok) {
      dispatch({ kind: 'reject', reason: check.reason })
      return
    }
    if (installed.includes(check.id)) {
      dispatch({ kind: 'reject', reason: ALREADY_HERE })
      return
    }
    awaited.current = signature
    dispatch({ kind: 'begin' })
    send({ kind: 'installExtension', source: check.id })
  }, [field.busy, field.value, installed, send, signature])

  return { ...field, change, submit }
}
