// Responsabilite : reglages groupes par theme d'apres le catalogue, et envoi des modifications au coeur.

import { useCallback, useMemo } from 'react'
import type { SettingValue, SettingView, UiRequest } from '../shared/contract'
import { definitionOf, SETTING_GROUPS, type SettingDefinition, type SettingGroup } from './setting-catalogue'

export interface SettingEntry {
  key: string
  value: SettingValue
  definition: SettingDefinition
}

export interface SettingSection {
  group: SettingGroup
  entries: SettingEntry[]
}

export interface SettingsController {
  sections: SettingSection[]
  update: (key: string, value: SettingValue) => void
}

/** Reglages geres par leur propre ecran, jamais listes dans la feuille de reglages. */
const HIDDEN = new Set(['tabs.folders', 'tabs.containers', 'profiles.names', 'sync.history', 'sync.mode'])

export function groupSettings(settings: SettingView[]): SettingSection[] {
  const entries: SettingEntry[] = settings.filter((item) => !HIDDEN.has(item.key)).map((item) => ({
    key: item.key,
    value: item.value,
    definition: definitionOf(item.key, item.value),
  }))
  return SETTING_GROUPS.map((group) => ({
    group,
    entries: entries.filter((entry) => entry.definition.group === group.id),
  })).filter((section) => section.entries.length > 0)
}

export function useSettings(send: (request: UiRequest) => void, settings: SettingView[]): SettingsController {
  const sections = useMemo(() => groupSettings(settings), [settings])
  const update = useCallback(
    (key: string, value: SettingValue): void => send({ kind: 'updateSetting', key, value }),
    [send],
  )
  return { sections, update }
}
