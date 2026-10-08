// Responsabilite : les fiches de formulaire (identite, contact, adresse) — lues dans le reglage synchronise
// `forms.cards`, creees, modifiees, supprimees. Jamais de mot de passe ni de carte bancaire : aucun champ pour.

import { useCallback, useMemo } from 'react'
import type { SettingView, UiRequest } from '../shared/contract'

const KEY = 'forms.cards'

export const FORM_FIELDS = [
  { key: 'givenName', label: 'Prénom', auto: 'given-name' },
  { key: 'familyName', label: 'Nom', auto: 'family-name' },
  { key: 'email', label: 'E-mail', auto: 'email' },
  { key: 'tel', label: 'Téléphone', auto: 'tel' },
  { key: 'organization', label: 'Société', auto: 'organization' },
  { key: 'street', label: 'Adresse', auto: 'street-address' },
  { key: 'postalCode', label: 'Code postal', auto: 'postal-code' },
  { key: 'city', label: 'Ville', auto: 'address-level2' },
  { key: 'country', label: 'Pays', auto: 'country-name' },
] as const

export type FormFieldKey = (typeof FORM_FIELDS)[number]['key']
export type FormFields = Partial<Record<FormFieldKey, string>>

export interface FormCard {
  id: string
  name: string
  fields: FormFields
}

export interface FormCards {
  list: FormCard[]
  create: () => void
  edit: (id: string, change: Partial<Omit<FormCard, 'id'>>) => void
  remove: (id: string) => void
}

function isCard(value: unknown): value is FormCard {
  if (typeof value !== 'object' || value === null) return false
  const card = value as Record<string, unknown> // forme verifiee champ par champ juste apres
  return typeof card.id === 'string' && typeof card.name === 'string'
    && typeof card.fields === 'object' && card.fields !== null
}

export function readCards(settings: SettingView[]): FormCard[] {
  const value = settings.find((s) => s.key === KEY)?.value
  if (value?.type !== 'text') return []
  try {
    const parsed: unknown = JSON.parse(value.value)
    return Array.isArray(parsed) ? parsed.filter(isCard) : []
  } catch {
    return []
  }
}

export function useFormCards(send: (request: UiRequest) => void, settings: SettingView[]): FormCards {
  const list = useMemo(() => readCards(settings), [settings])
  const save = useCallback((next: FormCard[]): void => {
    send({ kind: 'updateSetting', key: KEY, value: { type: 'text', value: JSON.stringify(next) } })
  }, [send])
  const create = useCallback((): void => {
    save([...list, { id: `f${Date.now().toString(36)}`, name: list.length === 0 ? 'Personnel' : 'Nouvelle fiche',
      fields: {} }])
  }, [list, save])
  const edit = useCallback((id: string, change: Partial<Omit<FormCard, 'id'>>): void => {
    save(list.map((card) => (card.id === id ? { ...card, ...change } : card)))
  }, [list, save])
  const remove = useCallback((id: string): void => save(list.filter((card) => card.id !== id)), [list, save])
  return { list, create, edit, remove }
}
