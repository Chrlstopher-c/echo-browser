// Responsabilite : presentation d'Echo au premier lancement (facon Arc) — quelques ecrans, puis le compte Echo a creer,
// ou passer. Termine : le reglage `onboarding.done` est pose et l'onglet devient un nouvel onglet.

import { AnimatePresence, motion } from 'framer-motion'
import { useState, type ReactElement } from 'react'
import { AccountForm } from '../account/account-form'
import type { AccountView, UiRequest } from '../shared/contract'
import { IconShield, IconSparkle, IconUser } from '../shared/design/icons'
import { PushButton } from '../shared/design/push-button'

interface Slide {
  icon: ReactElement
  title: string
  text: string
}

const SLIDES: Slide[] = [
  { icon: <IconSparkle size={30} />, title: 'Bienvenue dans Echo',
    text: 'Un navigateur rapide et sobre : les onglets inactifs s’endorment, la mémoire reste libre, la page est à vous.' },
  { icon: <IconUser size={30} />, title: 'Des profils comme des identités',
    text: 'Chaque profil a ses comptes, ses cookies et ses extensions. Travail, perso, projets : rien ne se mélange.' },
  { icon: <IconShield size={30} />, title: 'Un bouclier intégré',
    text: 'Publicités, traqueurs et pubs Twitch bloqués sans rien installer. Un clic pour l’écarter sur un site.' },
]

export interface WelcomePageProps {
  account: AccountView | null
  send: (request: UiRequest) => void
}

function finish(send: WelcomePageProps['send']): void {
  send({ kind: 'updateSetting', key: 'onboarding.done', value: { type: 'flag', value: true } })
  window.location.href = 'echo://ui/nouvel-onglet.html'
}

function AccountStep({ account, send }: { account: AccountView; send: WelcomePageProps['send'] }): ReactElement {
  if (account.email !== null) {
    return (
      <div className="flex flex-col items-center gap-4">
        <p className="text-[14px] text-ink">Connecté : <span className="font-medium">{account.email}</span></p>
        <PushButton tone="guard" onClick={() => finish(send)}>Commencer à naviguer</PushButton>
      </div>
    )
  }
  return (
    <div className="w-full max-w-sm">
      <AccountForm account={account} send={send} createFirst />
      <div className="mt-3 flex justify-center">
        <PushButton onClick={() => finish(send)}>Passer pour l’instant</PushButton>
      </div>
    </div>
  )
}

export function WelcomePage({ account, send }: WelcomePageProps): ReactElement {
  const [step, setStep] = useState(0)
  const withAccount = account !== null && account.available
  const total = SLIDES.length + (withAccount ? 1 : 0)
  const slide = SLIDES[step]
  const last = step === total - 1
  return (
    <div className="fond-espace flex h-screen flex-col items-center justify-center overflow-y-auto bg-shell px-6">
      <AnimatePresence mode="wait">
        <motion.div key={step} initial={{ opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -8 }}
          transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
          className="flex w-full max-w-lg flex-col items-center gap-5 rounded-tile bg-card px-10 py-12 text-center shadow-card">
          {slide !== undefined ? (
            <>
              <span className="grid size-16 place-items-center rounded-full bg-shell text-ink shadow-field">{slide.icon}</span>
              <h1 className="text-[24px] font-semibold text-ink">{slide.title}</h1>
              <p className="max-w-md text-[14px] leading-relaxed text-ink-muted">{slide.text}</p>
            </>
          ) : (
            <>
              <h1 className="text-[22px] font-semibold text-ink">Votre compte Echo</h1>
              <p className="max-w-md text-[13px] leading-relaxed text-ink-muted">
                Réglages, favoris, extensions et historique vous suivent sur toutes vos machines, chiffrés de bout en bout.
              </p>
              {account !== null && <AccountStep account={account} send={send} />}
            </>
          )}
        </motion.div>
      </AnimatePresence>
      <div className="mt-6 flex items-center gap-4">
        <div className="flex gap-1.5" aria-label={`Étape ${step + 1} sur ${total}`}>
          {Array.from({ length: total }, (_, i) => (
            <span key={i} className={`h-1.5 rounded-full transition-all ${i === step ? 'w-5 bg-ink' : 'w-1.5 bg-ink-faint/50'}`} />
          ))}
        </div>
        {slide !== undefined && (
          <PushButton tone="guard" onClick={() => (last ? finish(send) : setStep(step + 1))}>
            {last ? 'Commencer' : 'Suivant'}
          </PushButton>
        )}
        {step === 0 && <PushButton onClick={() => finish(send)}>Passer</PushButton>}
      </div>
    </div>
  )
}
