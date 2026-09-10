// Responsabilite : point d'entree du menu contextuel, page a part servie au-dessus du contenu.

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import '../shared/design/theme.css'
import { resolveBridge } from '../shared/core-bridge'
import { MenuPanel } from './menu-panel'
import { applyTheme, readPayload } from './read-payload'

const payload = readPayload(window.location.hash)
applyTheme(payload.theme)

const container = document.getElementById('racine')
if (container === null) throw new Error('Racine du menu introuvable dans le document.')

const { bridge } = resolveBridge()

createRoot(container).render(
  <StrictMode>
    <MenuPanel target={payload.target} send={bridge.send} />
  </StrictMode>,
)
