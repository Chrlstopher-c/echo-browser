// Responsabilite : une ligne de reglage — libelle, explication, et le controle qui va avec son type.

import type { ReactElement } from 'react'
import type { SettingValue } from '../shared/contract'
import { Stepper } from '../shared/design/stepper'
import { TextField } from '../shared/design/text-field'
import { Toggle } from '../shared/design/toggle'
import type { SettingEntry } from './use-settings'
import { ChoiceChips } from './choice-chips'

export interface SettingRowProps {
  entry: SettingEntry
  onChange: (value: SettingValue) => void
}

function Control({ entry, onChange }: SettingRowProps): ReactElement {
  const { value, definition } = entry
  switch (value.type) {
    case 'flag':
      return (
        <Toggle checked={value.value} label={definition.label}
          onChange={(next) => onChange({ type: 'flag', value: next })} />
      )
    case 'number': {
      const shape = definition.number ?? { min: 0, max: 10_000, step: 1, unit: '' }
      return (
        <Stepper value={value.value} min={shape.min} max={shape.max} step={shape.step} unit={shape.unit}
          label={definition.label} onChange={(next) => onChange({ type: 'number', value: next })} />
      )
    }
    case 'text':
      if (definition.choices !== undefined) {
        return (
          <ChoiceChips options={definition.choices} value={value.value} label={definition.label}
            onChange={(next) => onChange({ type: 'text', value: next })} />
        )
      }
      return (
        <TextField value={value.value} label={definition.label} placeholder={definition.placeholder ?? ''}
          mono={false} onCommit={(next) => onChange({ type: 'text', value: next })} />
      )
  }
}

export function SettingRow({ entry, onChange }: SettingRowProps): ReactElement {
  const stacked = entry.value.type === 'text'
  return (
    <div className={`flex gap-3 px-2 py-2 ${stacked ? 'flex-col' : 'items-center justify-between'}`}>
      <div className="min-w-0">
        <p className="text-[12.5px] text-ink">{entry.definition.label}</p>
        <p className="text-[11.5px] leading-snug text-ink-faint">{entry.definition.detail}</p>
      </div>
      <div className={stacked ? 'w-full' : 'shrink-0'}>
        <Control entry={entry} onChange={onChange} />
      </div>
    </div>
  )
}
