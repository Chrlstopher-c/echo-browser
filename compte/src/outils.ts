// Responsabilite : briques partagees du service — reponses JSON, empreintes, aleatoire, lecture des corps.

export interface Env {
  DB: D1Database
  /** Secret du serveur : sel fictif deterministe pour les adresses inconnues (pas d'enumeration des comptes). */
  SECRET_SEL: string
  /** Cle du tableau de bord des createurs ; absente, le tableau de bord est ferme. */
  ADMIN_KEY?: string
}

const CONTEXTES = new WeakMap<Request, ExecutionContext>()
const COMPTES = new WeakMap<Request, string>()

/** Retient le compte d'une requete authentifiee (pour l'usage par compte, compte a la fin de la requete). */
export function noterCompte(requete: Request, compte: string): void {
  COMPTES.set(requete, compte)
}

export function compteConnu(requete: Request): string | null {
  return COMPTES.get(requete) ?? null
}

/** Rattache une requete a son contexte d'execution, pour les travaux d'arriere-plan (`attendre`). */
export function lier(requete: Request, contexte: ExecutionContext): void {
  CONTEXTES.set(requete, contexte)
}

/** Laisse `travail` finir apres la reponse (compteurs, activite). */
export function attendre(requete: Request, travail: Promise<unknown>): void {
  const contexte = CONTEXTES.get(requete)
  if (contexte === undefined) void travail
  else contexte.waitUntil(travail)
}

export class Refus extends Error {
  constructor(readonly statut: number, message: string) {
    super(message)
  }
}

export function json(corps: unknown, statut = 200): Response {
  return new Response(JSON.stringify(corps), { status: statut, headers: { 'content-type': 'application/json' } })
}

export function base64(octets: ArrayBuffer | Uint8Array): string {
  const vue = octets instanceof Uint8Array ? octets : new Uint8Array(octets)
  let texte = ''
  for (const o of vue) texte += String.fromCharCode(o)
  return btoa(texte)
}

export function depuisBase64(texte: string): Uint8Array {
  const brut = atob(texte)
  return Uint8Array.from(brut, (c) => c.charCodeAt(0))
}

export function aleatoire(taille: number): Uint8Array {
  return crypto.getRandomValues(new Uint8Array(taille))
}

export async function sha256(...parties: Uint8Array[]): Promise<string> {
  const total = new Uint8Array(parties.reduce((n, p) => n + p.length, 0))
  let i = 0
  for (const p of parties) {
    total.set(p, i)
    i += p.length
  }
  return base64(await crypto.subtle.digest('SHA-256', total))
}

export async function hmac(secret: string, message: string): Promise<Uint8Array> {
  const cle = await crypto.subtle.importKey('raw', new TextEncoder().encode(secret), { name: 'HMAC', hash: 'SHA-256' },
    false, ['sign'])
  return new Uint8Array(await crypto.subtle.sign('HMAC', cle, new TextEncoder().encode(message)))
}

/** Compare deux chaines en temps constant (empreintes). */
export function egal(a: string, b: string): boolean {
  if (a.length !== b.length) return false
  let d = 0
  for (let i = 0; i < a.length; i++) d |= a.charCodeAt(i) ^ b.charCodeAt(i)
  return d === 0
}

export async function lireJson(requete: Request, max = 2_000_000): Promise<Record<string, unknown>> {
  const taille = Number(requete.headers.get('content-length') ?? '0')
  if (taille > max) throw new Refus(413, 'requete trop volumineuse')
  const texte = await requete.text()
  if (texte.length > max) throw new Refus(413, 'requete trop volumineuse')
  try {
    const valeur: unknown = JSON.parse(texte)
    if (typeof valeur !== 'object' || valeur === null || Array.isArray(valeur)) throw new Error()
    return valeur as Record<string, unknown>
  } catch {
    throw new Refus(400, 'corps JSON attendu')
  }
}

export function texte(champ: unknown, nom: string, max = 512): string {
  if (typeof champ !== 'string' || champ.length === 0 || champ.length > max) throw new Refus(400, `${nom} invalide`)
  return champ
}

export function normaliserEmail(champ: unknown): string {
  const email = texte(champ, 'email', 254).trim().toLowerCase()
  if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(email)) throw new Refus(400, 'email invalide')
  return email
}
