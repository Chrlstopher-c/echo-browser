// Responsabilite : les profils (identites facon Arc) — leur liste, leur nom et leur teinte, enregistres dans le reglage
// `profiles.list` (synchronise) ; creer, renommer, changer de teinte, reinitialiser, supprimer. Les cinq profils
// d'origine (une teinte chacun) restent ceux d'avant la liste, avec les noms de `profiles.names`.

import { useCallback, useMemo } from 'react'
import type { SettingView, UiRequest } from '../shared/contract'
import { HUES, isSpaceId, type SpaceId } from './space-palette'

const LIST_KEY = 'profiles.list'
const LEGACY_NAMES_KEY = 'profiles.names'
const LEGACY_KEY = 'profiles.legacy'
export const DEFAULT_PROFILE = 'graphite'

const DEFAULT_NAMES: Record<string, string> = {
  graphite: 'Personnel', sable: 'Travail', rose: 'Profil 3', foret: 'Profil 4', ardoise: 'Profil 5',
}

export interface ProfileEntry {
  id: string
  name: string
  hue: SpaceId
}

export interface ProfileNames {
  list: ProfileEntry[]
  nameOf: (id: string) => string
  hueOf: (id: string) => SpaceId
  rename: (id: string, name: string) => void
  setHue: (id: string, hue: SpaceId) => void
  /** Cree un profil et rend son identifiant. */
  create: (name: string) => string
  /** Sessions effacees tout de suite, le reste au prochain lancement. */
  reset: (id: string) => void
  remove: (id: string) => void
}

function textOf(settings: SettingView[], key: string): string | null {
  const raw = settings.find((item) => item.key === key)?.value
  return raw !== undefined && raw.type === 'text' ? raw.value : null
}

function legacyNames(settings: SettingView[]): Record<string, string> {
  try {
    const parsed: unknown = JSON.parse(textOf(settings, LEGACY_NAMES_KEY) ?? '{}')
    if (typeof parsed !== 'object' || parsed === null) return {}
    return Object.fromEntries(Object.entries(parsed).filter((e): e is [string, string] => typeof e[1] === 'string'))
  } catch (error) {
    console.warn('noms de profils illisibles', error)
    return {}
  }
}

function entryOf(value: unknown): ProfileEntry | null {
  if (typeof value !== 'object' || value === null) return null
  const fields = Object.fromEntries(Object.entries(value))
  const { id, name, hue } = fields
  if (typeof id !== 'string' || typeof name !== 'string' || typeof hue !== 'string' || !isSpaceId(hue)) return null
  return /^[A-Za-z0-9_-]{1,40}$/.test(id) ? { id, name, hue } : null
}

/** La liste enregistree, ou les cinq profils d'origine (avec leurs anciens noms). */
export function readProfiles(settings: SettingView[]): ProfileEntry[] {
  try {
    const parsed: unknown = JSON.parse(textOf(settings, LIST_KEY) || 'null')
    const list = Array.isArray(parsed) ? parsed.map(entryOf).filter((e): e is ProfileEntry => e !== null) : []
    if (list.some((entry) => entry.id === DEFAULT_PROFILE)) return list
  } catch (error) {
    console.warn('liste de profils illisible', error)
  }
  // Sans liste enregistree : le profil principal, plus les profils d'origine qui ont deja servi (une installation
  // neuve n'en montre qu'un, audit du 08/10 ; un utilisateur ancien retrouve les siens).
  const names = legacyNames(settings)
  const used = new Set((textOf(settings, LEGACY_KEY) ?? '').split(',').filter(Boolean))
  return HUES.filter((hue) => hue.id === DEFAULT_PROFILE || used.has(hue.id) || names[hue.id] !== undefined)
    .map((hue) => ({ id: hue.id, name: names[hue.id] ?? DEFAULT_NAMES[hue.id] ?? hue.name, hue: hue.id }))
}

export function useProfileNames(send: (request: UiRequest) => void, settings: SettingView[]): ProfileNames {
  const list = useMemo(() => readProfiles(settings), [settings])
  const save = useCallback((next: ProfileEntry[]): void => {
    send({ kind: 'updateSetting', key: LIST_KEY, value: { type: 'text', value: JSON.stringify(next) } })
  }, [send])
  const nameOf = useCallback((id: string): string => list.find((p) => p.id === id)?.name ?? id, [list])
  const hueOf = useCallback((id: string): SpaceId => list.find((p) => p.id === id)?.hue ?? 'graphite', [list])
  const edit = useCallback((id: string, change: Partial<ProfileEntry>): void => {
    save(list.map((p) => (p.id === id ? { ...p, ...change } : p)))
  }, [list, save])
  const create = useCallback((name: string): string => {
    const id = `p${Date.now().toString(36)}`
    const hue = HUES[list.length % HUES.length]?.id ?? 'graphite'
    save([...list, { id, name: name.trim() || `Profil ${list.length + 1}`, hue }])
    return id
  }, [list, save])
  const remove = useCallback((id: string): void => {
    if (id === DEFAULT_PROFILE) return
    send({ kind: 'profileForget', id, delete: true })
    save(list.filter((p) => p.id !== id))
  }, [list, save, send])
  return {
    list, nameOf, hueOf, create, remove,
    rename: (id, name) => edit(id, { name: name.trim() || nameOf(id) }),
    setHue: (id, hue) => edit(id, { hue }),
    reset: (id) => { if (id !== DEFAULT_PROFILE) send({ kind: 'profileForget', id, delete: false }) },
  }
}
