// Responsabilite : Reglages → Formulaires — les fiches que le clic droit « Remplir : … » verse dans les champs vides
// d'une page. Chaque champ s'enregistre en quittant la saisie (une ecriture par champ, pas par touche).

import { useState, type ReactElement } from 'react'
import { ConfirmStrip } from '../shared/design/confirm-strip'
import { IconPlus } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'
import { SectionLabel } from '../shared/design/section-label'
import { FORM_FIELDS, type FormCard, type FormCards } from './form-cards'

const INPUT = 'h-7 min-w-0 w-full rounded-md bg-field px-2 text-[12px] text-ink shadow-field outline-none'

function Field({ label, auto, value, onSave }: { label: string; auto: string; value: string;
  onSave: (value: string) => void }): ReactElement {
  const [draft, setDraft] = useState(value)
  return (
    <label className="flex min-w-0 flex-col gap-0.5">
      <span className="px-0.5 text-[10.5px] text-ink-faint">{label}</span>
      <input value={draft} autoComplete={auto} aria-label={label} onChange={(event) => setDraft(event.target.value)}
        onBlur={() => draft !== value && onSave(draft.trim())} className={INPUT} />
    </label>
  )
}

function CardEditor({ card, cards }: { card: FormCard; cards: FormCards }): ReactElement {
  const [name, setName] = useState(card.name)
  const [asking, setAsking] = useState(false)
  if (asking) {
    return <ConfirmStrip question={`Supprimer la fiche « ${card.name} » ?`} confirmLabel="Supprimer"
      onCancel={() => setAsking(false)} onConfirm={() => cards.remove(card.id)} />
  }
  return (
    <div className="flex flex-col gap-2 rounded-tile bg-card p-2.5 shadow-card" data-fiche={card.id}>
      <div className="flex items-center gap-2">
        <input value={name} aria-label="Nom de la fiche" onChange={(event) => setName(event.target.value)}
          onBlur={() => name.trim() !== card.name && cards.edit(card.id, { name: name.trim() || card.name })}
          className={`${INPUT} flex-1 font-medium`} />
        <PushButton tone="danger" onClick={() => setAsking(true)}>Supprimer</PushButton>
      </div>
      <div className="grid grid-cols-2 gap-x-2 gap-y-1.5">
        {FORM_FIELDS.map((field) => (
          <Field key={field.key} label={field.label} auto={field.auto} value={card.fields[field.key] ?? ''}
            onSave={(value) => cards.edit(card.id, { fields: { ...card.fields, [field.key]: value } })} />
        ))}
      </div>
    </div>
  )
}

export function FormsSection({ cards }: { cards: FormCards }): ReactElement {
  return (
    <section>
      <SectionLabel aside={
        <PushButton icon={<IconPlus size={12} />} onClick={cards.create}>Nouvelle fiche</PushButton>
      }>Formulaires</SectionLabel>
      <div className="flex flex-col gap-2 px-1">
        {cards.list.map((card) => <CardEditor key={card.id} card={card} cards={cards} />)}
      </div>
      <p className="px-2 pt-1 text-[11px] leading-snug text-ink-faint">
        En entrant dans un champ reconnu (nom, e-mail, adresse…), la barre propose vos fiches ; un clic complète les
        champs vides. Aussi au clic droit dans un champ. Jamais de mot de passe ni de carte bancaire. Les fiches suivent votre compte Echo, chiffrées.
      </p>
    </section>
  )
}
