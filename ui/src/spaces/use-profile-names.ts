// Responsabilite : les noms des profils (les pastilles du bas de la barre), enregistres dans `profiles.names`.

import { useCallback, useMemo } from 'react'
import type { SettingView, UiRequest } from '../shared/contract'
import type { SpaceId } from './space-palette'

const KEY = 'profiles.names'

const DEFAULT_NAMES: Record<string, string> = {
  graphite: 'Personnel',
  sable: 'Travail',
  rose: 'Profil 3',
  foret: 'Profil 4',
  ardoise: 'Profil 5',
}

export interface ProfileNames {
  nameOf: (id: SpaceId) => string
  rename: (id: SpaceId, name: string) => void
}

function parse(settings: SettingView[]): Record<string, string> {
  const raw = settings.find((item) => item.key === KEY)?.value
  if (raw === undefined || raw.type !== 'text') return {}
  try {
    const parsed: unknown = JSON.parse(raw.value)
    if (typeof parsed !== 'object' || parsed === null) return {}
    return Object.fromEntries(Object.entries(parsed).filter((entry): entry is [string, string] => typeof entry[1] === 'string'))
  } catch (error) {
    console.warn('noms de profils illisibles', error)
    return {}
  }
}

export function useProfileNames(send: (request: UiRequest) => void, settings: SettingView[]): ProfileNames {
  const names = useMemo(() => parse(settings), [settings])
  const nameOf = useCallback((id: SpaceId): string => names[id] ?? DEFAULT_NAMES[id] ?? id, [names])
  const rename = useCallback(
    (id: SpaceId, name: string): void => {
      const next = { ...names, [id]: name.trim() || (DEFAULT_NAMES[id] ?? id) }
      send({ kind: 'updateSetting', key: KEY, value: { type: 'text', value: JSON.stringify(next) } })
    },
    [names, send],
  )
  return { nameOf, rename }
}
