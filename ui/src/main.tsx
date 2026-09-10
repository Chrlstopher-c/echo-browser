// Responsabilite : point d'entree de l'interface.

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './shared/design/theme.css'
import { BrowserChrome } from './shell/browser-chrome'

const container = document.getElementById('racine')
if (container === null) throw new Error("Racine d'interface introuvable dans le document.")

createRoot(container).render(
  <StrictMode>
    <BrowserChrome />
  </StrictMode>,
)
