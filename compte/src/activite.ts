// Responsabilite : ce que le service sait de l'usage, sans rien du contenu — derniere activite d'un compte (et la
// version d'Echo), compteurs de requetes par jour. Ecrit en arriere-plan : une erreur ici ne fait pas echouer la
// requete.

import type { Env } from './outils'

/** Une activite n'est reecrite qu'au-dela de ce delai (une ecriture par heure et par compte au plus). */
const PAS_ACTIVITE_MS = 3600 * 1000
const VERSION = /^[0-9A-Za-z.+-]{1,32}$/

export function jour(ms = Date.now()): string {
  return new Date(ms).toISOString().slice(0, 10)
}

/** La route sans ses parametres : `PUT /v1/coffre/:type`. */
export function routeDe(requete: Request): string {
  const { pathname } = new URL(requete.url)
  const route = pathname.replace(/^\/v1\/coffre\/[^/]+$/, '/v1/coffre/:type').replace(/^\/v1\/admin\/comptes\/[^/]+/,
    '/v1/admin/comptes/:id')
  return `${requete.method} ${route}`
}

export async function compter(env: Env, requete: Request, statut: number): Promise<void> {
  try {
    await env.DB.prepare(`INSERT INTO compteurs (jour, cle, n) VALUES (?, ?, 1)
      ON CONFLICT (jour, cle) DO UPDATE SET n = n + 1`)
      .bind(jour(), statut === 404 ? 'introuvable 404' : `${routeDe(requete)} ${statut}`).run()
  } catch (erreur) {
    console.error('compteur non ecrit', erreur)
  }
}

export async function noterActivite(env: Env, compte: string, requete: Request): Promise<void> {
  const brute = requete.headers.get('x-echo-version') ?? ''
  const version = VERSION.test(brute) ? brute : null
  const maintenant = Date.now()
  try {
    await env.DB.prepare(`INSERT INTO activite (compte, vu_le, version) VALUES (?, ?, ?)
      ON CONFLICT (compte) DO UPDATE SET vu_le = excluded.vu_le, version = COALESCE(excluded.version, activite.version)
      WHERE activite.vu_le < ? OR activite.version IS NOT excluded.version`)
      .bind(compte, maintenant, version, maintenant - PAS_ACTIVITE_MS).run()
  } catch (erreur) {
    console.error('activite non notee', erreur)
  }
}
