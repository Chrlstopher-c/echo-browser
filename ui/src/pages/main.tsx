// Responsabilite : point d'entree des pages pleine largeur (reglages, bibliotheque) ouvertes dans un onglet.

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import '../shared/design/theme.css'
import { PagesApp } from './pages-app'

// La fenetre d'Echo est transparente (angles arrondis) : une page pleine largeur peint son propre fond.
document.documentElement.style.background = 'var(--color-shell, #222326)'
document.body.style.background = 'var(--color-shell, #222326)'

const container = document.getElementById('racine')
if (container === null) throw new Error('Racine des pages introuvable dans le document.')

createRoot(container).render(
  <StrictMode>
    <PagesApp />
  </StrictMode>,
)
