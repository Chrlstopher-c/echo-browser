// Responsabilite : lire ce que l'utilisateur colle dans le champ d'installation, et le refuser
// avant tout envoi au coeur. Un identifiant du catalogue Chrome fait exactement 32 lettres de a a p.

/** Identifiant seul, du debut a la fin de la saisie. */
const LONE_ID = /^[a-p]{32}$/

/** Identifiant isole au milieu d'un texte : borne des deux cotes pour ne pas couper une suite plus longue. */
const EMBEDDED_ID = /(?:^|[^a-p])([a-p]{32})(?![a-p])/

/** Hotes du catalogue Chrome reconnus. Le sous-domaine exact suffit, pas de correspondance partielle. */
const STORE_HOSTS = new Set(['chromewebstore.google.com', 'chrome.google.com'])

export type SourceCheck = { ok: true; source: string; id: string } | { ok: false; reason: string }

const MALFORMED = 'Collez une adresse du Chrome Web Store, ou un identifiant de 32 lettres (a à p).'
const FOREIGN_HOST = 'Cette adresse ne vient pas du Chrome Web Store.'
const NO_ID_IN_URL = "Aucun identifiant d'extension dans cette adresse."

function idWithin(text: string): string | null {
  const found = EMBEDDED_ID.exec(text)
  return found?.[1] ?? null
}

function readStoreUrl(value: string): SourceCheck {
  let host: string
  try {
    host = new URL(value).hostname.toLowerCase()
  } catch {
    return { ok: false, reason: MALFORMED }
  }
  if (!STORE_HOSTS.has(host)) return { ok: false, reason: FOREIGN_HOST }
  const id = idWithin(value)
  if (id === null) return { ok: false, reason: NO_ID_IN_URL }
  return { ok: true, source: value, id }
}

/** Valide une saisie et en extrait l'identifiant. Aucune requete n'est emise sur un refus.
 *  Un identifiant n'est extrait que d'une adresse : ailleurs, il doit occuper toute la saisie,
 *  faute de quoi une suite de 33 lettres passerait pour un identifiant de 32. */
export function readExtensionSource(input: string): SourceCheck {
  const value = input.trim()
  if (value.length === 0) return { ok: false, reason: MALFORMED }
  if (LONE_ID.test(value)) return { ok: true, source: value, id: value }
  if (/^https?:\/\//i.test(value)) return readStoreUrl(value)
  if (value.includes('/')) return readStoreUrl(`https://${value}`)
  return { ok: false, reason: MALFORMED }
}
