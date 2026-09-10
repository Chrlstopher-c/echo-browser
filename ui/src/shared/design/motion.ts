// Responsabilite : rythme du mouvement — une seule courbe, des durees courtes, jamais de rebond.

import type { Transition } from 'framer-motion'

export const CHROME_EASE: [number, number, number, number] = [0.22, 0.61, 0.36, 1]

/** Apparition ou disparition d'un element de liste. */
export const QUICK: Transition = { duration: 0.16, ease: CHROME_EASE }

/** Ouverture d'un panneau, glissement du fond actif. */
export const PANEL: Transition = { duration: 0.2, ease: CHROME_EASE }

/** Suivi d'une valeur continue (progression) : assez lent pour lisser, assez court pour rester vrai. */
export const TRACK: Transition = { duration: 0.24, ease: 'linear' }

/** Repli et depli de la barre, en secondes. */
export const COLLAPSE_SECONDS = 0.22
