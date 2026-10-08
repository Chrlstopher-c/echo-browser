// Responsabilite : comptes et sessions. Le serveur ne voit jamais le mot de passe : il recoit une cle d'acces derivee
// sur la machine (PBKDF2 + HKDF) et n'en garde qu'une empreinte salee. Jetons de session : empreinte seulement.

import {
  aleatoire, attendre, base64, depuisBase64, egal, hmac, json, lireJson, noterCompte, normaliserEmail, Refus, sha256,
  texte, type Env,
} from './outils'
import { noterActivite, versionDe } from './activite'

const ITERATIONS_PAR_DEFAUT = 600_000
const DUREE_SESSION_MS = 90 * 24 * 3600 * 1000
const FENETRE_ECHECS_MS = 15 * 60 * 1000
const MAX_ECHECS = 10

interface Compte {
  id: string
  empreinte_acces: string
  sel_serveur: string
  sel_client: string
  iterations: number
}

function cleAcces(champ: unknown): Uint8Array {
  const cle = depuisBase64(texte(champ, 'cleAcces', 64))
  if (cle.length !== 32) throw new Refus(400, 'cleAcces invalide')
  return cle
}

async function ouvrirSession(requete: Request, env: Env, compte: string): Promise<string> {
  const jeton = aleatoire(32)
  const empreinte = await sha256(jeton)
  const maintenant = Date.now()
  await env.DB.batch([
    env.DB.prepare('INSERT INTO sessions (empreinte_jeton, compte, expire_le) VALUES (?, ?, ?)')
      .bind(empreinte, compte, maintenant + DUREE_SESSION_MS),
    env.DB.prepare('INSERT INTO machines (empreinte_jeton, compte, cree_le, vu_le, version) VALUES (?, ?, ?, ?, ?)')
      .bind(empreinte, compte, maintenant, maintenant, versionDe(requete)),
  ])
  noterCompte(requete, compte)
  return base64(jeton)
}

/** Le sel et le cout de derivation d'un compte ; pour une adresse inconnue, des valeurs fictives stables. */
export async function sel(requete: Request, env: Env): Promise<Response> {
  const email = normaliserEmail((await lireJson(requete)).email)
  const compte = await env.DB.prepare('SELECT sel_client, iterations FROM comptes WHERE email = ?').bind(email)
    .first<{ sel_client: string; iterations: number }>()
  if (compte !== null) return json({ sel: compte.sel_client, iterations: compte.iterations })
  const fictif = (await hmac(env.SECRET_SEL, email)).slice(0, 16)
  return json({ sel: base64(fictif), iterations: ITERATIONS_PAR_DEFAUT })
}

export async function inscription(requete: Request, env: Env): Promise<Response> {
  const corps = await lireJson(requete)
  const email = normaliserEmail(corps.email)
  const cle = cleAcces(corps.cleAcces)
  const selClient = texte(corps.sel, 'sel', 64)
  if (depuisBase64(selClient).length !== 16) throw new Refus(400, 'sel invalide')
  const iterations = Number(corps.iterations)
  if (!Number.isInteger(iterations) || iterations < 100_000 || iterations > 5_000_000) throw new Refus(400, 'iterations invalides')
  const existe = await env.DB.prepare('SELECT 1 FROM comptes WHERE email = ?').bind(email).first()
  if (existe !== null) throw new Refus(409, 'un compte existe deja pour cette adresse')
  const id = crypto.randomUUID()
  const selServeur = aleatoire(16)
  await env.DB.prepare(
    'INSERT INTO comptes (id, email, empreinte_acces, sel_serveur, sel_client, iterations, cree_le) VALUES (?, ?, ?, ?, ?, ?, ?)',
  ).bind(id, email, await sha256(selServeur, cle), base64(selServeur), selClient, iterations, Date.now()).run()
  return json({ jeton: await ouvrirSession(requete, env, id) }, 201)
}

export async function connexion(requete: Request, env: Env): Promise<Response> {
  const corps = await lireJson(requete)
  const email = normaliserEmail(corps.email)
  const cle = cleAcces(corps.cleAcces)
  const depuis = Date.now() - FENETRE_ECHECS_MS
  const echecs = await env.DB.prepare('SELECT COUNT(*) AS n FROM echecs WHERE email = ? AND le > ?').bind(email, depuis)
    .first<{ n: number }>()
  if ((echecs?.n ?? 0) >= MAX_ECHECS) throw new Refus(429, 'trop de tentatives, reessayer dans un quart d\'heure')
  const compte = await env.DB.prepare('SELECT * FROM comptes WHERE email = ?').bind(email).first<Compte>()
  const attendu = compte === null ? '' : compte.empreinte_acces
  const recu = compte === null ? 'x' : await sha256(depuisBase64(compte.sel_serveur), cle)
  if (compte === null || !egal(attendu, recu)) {
    await env.DB.prepare('INSERT INTO echecs (email, le) VALUES (?, ?)').bind(email, Date.now()).run()
    throw new Refus(401, 'adresse ou mot de passe incorrect')
  }
  return json({ jeton: await ouvrirSession(requete, env, compte.id) })
}

/** Le compte d'une requete authentifiee (en-tete `Authorization: Bearer <jeton>`). */
export async function compteDe(requete: Request, env: Env): Promise<string> {
  const entete = requete.headers.get('authorization') ?? ''
  const jeton = entete.startsWith('Bearer ') ? entete.slice(7) : ''
  if (jeton.length === 0) throw new Refus(401, 'session absente')
  const empreinte = await sha256(depuisBase64(jeton))
  const session = await env.DB.prepare('SELECT compte, expire_le FROM sessions WHERE empreinte_jeton = ?')
    .bind(empreinte).first<{ compte: string; expire_le: number }>()
  if (session === null || session.expire_le < Date.now()) throw new Refus(401, 'session expiree')
  noterCompte(requete, session.compte)
  attendre(requete, noterActivite(env, session.compte, empreinte, requete))
  return session.compte
}

export async function deconnexion(requete: Request, env: Env): Promise<Response> {
  const entete = requete.headers.get('authorization') ?? ''
  const jeton = entete.startsWith('Bearer ') ? entete.slice(7) : ''
  if (jeton.length > 0) {
    const empreinte = await sha256(depuisBase64(jeton))
    await env.DB.batch([
      env.DB.prepare('DELETE FROM sessions WHERE empreinte_jeton = ?').bind(empreinte),
      env.DB.prepare('DELETE FROM machines WHERE empreinte_jeton = ?').bind(empreinte),
    ])
  }
  return new Response(null, { status: 204 })
}

export async function supprimer(requete: Request, env: Env): Promise<Response> {
  await effacerCompte(env, await compteDe(requete, env))
  return new Response(null, { status: 204 })
}

/** Efface un compte et tout ce que le service en garde. */
export async function effacerCompte(env: Env, compte: string): Promise<void> {
  await env.DB.batch([
    env.DB.prepare('DELETE FROM activite WHERE compte = ?').bind(compte),
    env.DB.prepare('DELETE FROM admins WHERE compte = ?').bind(compte),
    env.DB.prepare('DELETE FROM machines WHERE compte = ?').bind(compte),
    env.DB.prepare('DELETE FROM usage WHERE compte = ?').bind(compte),
    env.DB.prepare('DELETE FROM coffre WHERE compte = ?').bind(compte),
    env.DB.prepare('DELETE FROM sessions WHERE compte = ?').bind(compte),
    env.DB.prepare('DELETE FROM comptes WHERE id = ?').bind(compte),
  ])
}

/** Ce que la machine connectee doit savoir de son compte : s'il ouvre l'administration. */
export async function moi(requete: Request, env: Env): Promise<Response> {
  const compte = await compteDe(requete, env)
  const admin = await env.DB.prepare('SELECT 1 FROM admins WHERE compte = ?').bind(compte).first()
  return json({ admin: admin !== null })
}
