// Responsabilite : les dossiers d'onglets. La liste (nom, plie ou deplie) vit dans le reglage `tabs.folders` ;
// le rangement de chaque onglet, lui, est porte par l'onglet et survit aux relances avec la session.

import { useCallback, useMemo, useState } from 'react'
import type { SettingView, TabId, TabView, UiRequest } from '../shared/contract'

export interface Folder {
  id: string
  name: string
  collapsed: boolean
  /** Profil du dossier ; absent pour les dossiers d'avant les profils separes (profil principal). */
  space?: string
}

export interface FolderActions {
  folders: Folder[]
  /** Dossier dont le nom est en cours de saisie. */
  editing: string | null
  edit: (id: string | null) => void
  create: (name?: string, tabId?: TabId) => void
  rename: (id: string, name: string) => void
  toggle: (id: string) => void
  /** Supprime le dossier ; ses onglets restent ouverts, simplement libres. */
  remove: (id: string, members: TabView[]) => void
  /** Ferme le dossier et tous ses onglets. */
  closeAll: (id: string, members: TabView[]) => void
  assign: (tabId: TabId, folder: string | null) => void
}

const KEY = 'tabs.folders'
const DEFAULT_SPACE = 'graphite'
const DEFAULT_NAME = 'Nouveau dossier'

function isFolder(value: unknown): value is Folder {
  if (typeof value !== 'object' || value === null) return false
  const candidate = value as Record<string, unknown>
  return typeof candidate.id === 'string' && typeof candidate.name === 'string'
}

export function parseFolders(settings: SettingView[]): Folder[] {
  const raw = settings.find((item) => item.key === KEY)?.value
  if (raw === undefined || raw.type !== 'text') return []
  try {
    const parsed: unknown = JSON.parse(raw.value)
    if (!Array.isArray(parsed)) return []
    return parsed.filter(isFolder).map((item) => ({
      id: item.id, name: item.name, collapsed: item.collapsed === true,
      ...(typeof item.space === 'string' ? { space: item.space } : {}),
    }))
  } catch (error) {
    console.warn('dossiers illisibles', error)
    return []
  }
}

function patch(folders: Folder[], id: string, change: Partial<Folder>): Folder[] {
  return folders.map((folder) => (folder.id === id ? { ...folder, ...change } : folder))
}

interface Deps {
  folders: Folder[]
  space: string
  editing: string | null
  edit: (id: string | null) => void
  save: (next: Folder[]) => void
  assign: (id: TabId, folder: string | null) => void
  send: (request: UiRequest) => void
}

function buildActions({ folders, space, editing, edit, save, assign, send }: Deps): FolderActions {
  const toggle = (id: string): void =>
    save(patch(folders, id, { collapsed: !(folders.find((f) => f.id === id)?.collapsed ?? false) }))
  return {
    folders,
    editing,
    edit,
    create: (name, tabId) => {
      const id = `d${Date.now().toString(36)}`
      save([...folders, { id, name: name ?? DEFAULT_NAME, collapsed: false, space }])
      if (tabId !== undefined) assign(tabId, id)
      edit(id)
    },
    rename: (id, name) => save(patch(folders, id, { name: name.trim() || DEFAULT_NAME })),
    toggle,
    remove: (id, members) => {
      members.forEach((tab) => assign(tab.id, null))
      save(folders.filter((f) => f.id !== id))
    },
    closeAll: (id, members) => {
      members.forEach((tab) => send({ kind: 'closeTab', id: tab.id }))
      save(folders.filter((f) => f.id !== id))
    },
    assign,
  }
}

/** Les dossiers du profil `space` seulement : ceux des autres profils restent enregistres, mais invisibles ici. */
export function useFolders(send: (request: UiRequest) => void, settings: SettingView[], space: string): FolderActions {
  const all = useMemo(() => parseFolders(settings), [settings])
  const mine = useCallback((folder: Folder): boolean => (folder.space ?? DEFAULT_SPACE) === space, [space])
  const folders = useMemo(() => all.filter(mine), [all, mine])
  const [editing, edit] = useState<string | null>(null)
  const save = useCallback(
    (next: Folder[]): void => {
      const kept = [...all.filter((folder) => !mine(folder)), ...next]
      send({ kind: 'updateSetting', key: KEY, value: { type: 'text', value: JSON.stringify(kept) } })
    },
    [send, all, mine],
  )
  const assign = useCallback(
    (id: TabId, folder: string | null): void => send({ kind: 'setTabFolder', id, folder }),
    [send],
  )
  return useMemo(
    () => buildActions({ folders, space, editing, edit, save, assign, send }),
    [folders, space, editing, save, assign, send],
  )
}
