// Responsabilite : garde d'accessibilite — tout texte (encre, encre attenuee, encre pale) atteint 4,5:1 sur les
// surfaces ou il se pose, pour chaque teinte d'espace, en clair et en sombre. `bun scripts/contrast.ts` (CI).

import { buildSpace, HUES } from '../src/spaces/space-palette'

const MINIMUM = 4.5

function luminance(hex: string): number {
  const channels = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
  const [r, g, b] = channels.map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4))
  return 0.2126 * (r ?? 0) + 0.7152 * (g ?? 0) + 0.0722 * (b ?? 0)
}

function ratio(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x)
  return ((hi ?? 0) + 0.05) / ((lo ?? 0) + 0.05)
}

let failures = 0
for (const hue of HUES) {
  for (const scheme of ['light', 'dark'] as const) {
    const { tokens } = buildSpace(hue.id, scheme)
    for (const ink of ['ink', 'inkMuted', 'inkFaint'] as const) {
      for (const surface of ['shell', 'card', 'hover', 'field'] as const) {
        const value = ratio(tokens[ink], tokens[surface])
        if (value < MINIMUM) {
          failures += 1
          console.error(`${hue.id} ${scheme} : ${ink} sur ${surface} = ${value.toFixed(2)}:1`)
        }
      }
    }
  }
}
if (failures > 0) process.exit(1)
console.log(`contrastes ≥ ${MINIMUM}:1 pour ${HUES.length} teintes, clair et sombre`)
