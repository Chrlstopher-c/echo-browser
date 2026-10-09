// Responsabilite : premier lancement — un accueil qui configure (theme, moteur de recherche, import depuis un autre
// navigateur), puis le compte Echo a creer ou passer. Termine : `onboarding.done` est pose et l'onglet devient un
// nouvel onglet, sans laisser l'accueil dans son historique.

import { AnimatePresence, motion } from 'framer-motion'
import { useState, type ReactElement } from 'react'
import { AccountForm } from '../account/account-form'
import { ImportSection } from '../import/import-section'
import { ChoiceChips } from '../settings/choice-chips'
import { SEARCH_ENGINES } from '../settings/setting-catalogue'
import type { AccountView, ImportSourceView, SettingView, UiRequest } from '../shared/contract'
import { IconSparkle } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'
import { Segmented, type Segment } from '../shared/design/segmented'
import { readSchemeChoice, writeSchemeChoice, type SchemeChoice } from '../spaces/use-space'

export interface WelcomePageProps {
  account: AccountView | null
  settings: SettingView[]
  importSources: ImportSourceView[] | null
  send: (request: UiRequest) => void
}

type Step = 'bienvenue' | 'theme' | 'moteur' | 'import' | 'compte'

const SCHEMES: Array<Segment<SchemeChoice>> = [
  { id: 'light', label: 'Clair' }, { id: 'dark', label: 'Sombre' }, { id: 'system', label: 'Système' },
]

function finish(send: WelcomePageProps['send']): void {
  send({ kind: 'updateSetting', key: 'onboarding.done', value: { type: 'flag', value: true } })
  window.location.replace('echo://ui/nouvel-onglet.html')
}

function Heading({ title, text }: { title: string; text: string }): ReactElement {
  return (
    <>
      <h1 className="text-[24px] font-semibold text-ink">{title}</h1>
      <p className="max-w-md text-[14px] leading-relaxed text-ink-muted">{text}</p>
    </>
  )
}

function ThemeStep(): ReactElement {
  const [choice, setChoice] = useState<SchemeChoice>(readSchemeChoice)
  return (
    <>
      <Heading title="Clair ou sombre ?" text="Echo suit votre choix partout : barre, réglages, pages des sites." />
      <div className="w-full max-w-xs">
        <Segmented name="accueil-theme" segments={SCHEMES} value={choice}
          onChange={(next) => { writeSchemeChoice(next); setChoice(next) }} />
      </div>
    </>
  )
}

function EngineStep({ settings, send }: Pick<WelcomePageProps, 'settings' | 'send'>): ReactElement {
  const raw = settings.find((s) => s.key === 'search.engine')?.value
  const current = raw !== undefined && raw.type === 'text' ? raw.value : 'google'
  return (
    <>
      <Heading title="Votre moteur de recherche"
        text="Ce que vous tapez dans l’adresse et qui n’est pas un site y est cherché. Modifiable dans les Réglages." />
      <ChoiceChips options={SEARCH_ENGINES} value={current} label="Moteur de recherche"
        onChange={(next) => send({
          kind: 'updateSetting', key: 'search.engine', value: { type: 'text', value: next },
        })} />
    </>
  )
}

function AccountStep({ account, send }: { account: AccountView; send: WelcomePageProps['send'] }): ReactElement {
  if (account.email !== null) {
    return <p className="text-[14px] text-ink">Connecté : <span className="font-medium">{account.email}</span></p>
  }
  return (
    <>
      <Heading title="Votre compte Echo"
        text="Réglages, favoris, extensions et historique vous suivent sur vos machines, chiffrés de bout en bout." />
      <div className="w-full max-w-sm"><AccountForm account={account} send={send} createFirst /></div>
    </>
  )
}

function StepBody({ step, props }: { step: Step; props: WelcomePageProps }): ReactElement {
  switch (step) {
    case 'bienvenue':
      return (
        <>
          <span className="grid size-16 place-items-center rounded-full bg-shell text-ink shadow-field">
            <IconSparkle size={30} />
          </span>
          <Heading title="Bienvenue dans Echo" text={'Un navigateur rapide et sobre : publicités et traqueurs bloqués, '
            + 'onglets inactifs endormis, un profil par identité. Trois réglages et c’est parti.'} />
        </>
      )
    case 'theme':
      return <ThemeStep />
    case 'moteur':
      return <EngineStep settings={props.settings} send={props.send} />
    case 'import':
      return (
        <>
          <Heading title="Reprendre vos favoris"
            text={'Echo peut reprendre les favoris et l’historique d’un navigateur déjà installé. '
              + 'Rien n’est modifié chez lui.'} />
          <div className="w-full max-w-sm"><ImportSection sources={props.importSources} send={props.send} /></div>
        </>
      )
    case 'compte':
      return props.account === null ? <></> : <AccountStep account={props.account} send={props.send} />
  }
}

export function WelcomePage(props: WelcomePageProps): ReactElement {
  const { account, send } = props
  const steps: Step[] = ['bienvenue', 'theme', 'moteur', 'import',
    ...(account !== null && account.available ? ['compte' as const] : [])]
  const [index, setIndex] = useState(0)
  const step = steps[index] ?? 'bienvenue'
  const last = index === steps.length - 1
  return (
    <div className="fond-espace flex h-screen flex-col items-center justify-center overflow-y-auto bg-shell px-6">
      <AnimatePresence mode="wait">
        <motion.div key={step} initial={{ opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -8 }}
          transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }} data-step={step}
          className="flex w-full max-w-lg flex-col items-center gap-5 rounded-tile bg-card px-10 py-12 text-center
            shadow-card">
          <StepBody step={step} props={props} />
        </motion.div>
      </AnimatePresence>
      <div className="mt-6 flex items-center gap-3">
        {index > 0 && <PushButton onClick={() => setIndex(index - 1)}>Précédent</PushButton>}
        <div className="flex gap-1.5" aria-label={`Étape ${index + 1} sur ${steps.length}`}>
          {steps.map((s, i) => (
            <span key={s}
              className={`h-1.5 rounded-full transition-all ${i === index ? 'w-5 bg-ink' : 'w-1.5 bg-ink-faint/50'}`} />
          ))}
        </div>
        <PushButton tone="guard" onClick={() => (last ? finish(send) : setIndex(index + 1))}>
          {last ? 'Commencer' : 'Suivant'}
        </PushButton>
        {!last && <PushButton onClick={() => finish(send)}>Passer</PushButton>}
      </div>
    </div>
  )
}
