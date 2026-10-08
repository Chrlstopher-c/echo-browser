// Responsabilite : ce que le service sait de l'usage, sans rien du contenu — derniere activite d'un compte (et la
// version d'Echo), compteurs de requetes par jour. Ecrit en arriere-plan : une erreur ici ne fait pas echouer la
// requete.

import { compteConnu, type Env } from './outils'

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

/** L'action d'une requete authentifiee, pour l'usage par compte : `synchro`, `ecriture:favoris`, `connexion`… */
export function actionDe(requete: Request): string {
  const { pathname } = new URL(requete.url)
  const type = /^\/v1\/coffre\/([a-z0-9-]{1,32})$/.exec(pathname)?.[1]
  if (type !== undefined) return `ecriture:${type}`
  if (pathname === '/v1/coffre') return 'synchro'
  if (pathname.startsWith('/v1/admin/')) return 'administration'
  return pathname.replace(/^\/v1\//, '').slice(0, 32)
}

export function versionDe(requete: Request): string | null {
  const brute = requete.headers.get('x-echo-version') ?? ''
  return VERSION.test(brute) ? brute : null
}

/** Une seule ecriture par requete (offre gratuite : 100 000 lignes ecrites par jour) : l'usage du compte quand il est
 * connu et que la requete a reussi, sinon le compteur de la route et de son statut. */
export async function compter(env: Env, requete: Request, statut: number): Promise<void> {
  const compte = compteConnu(requete)
  const ligne = compte !== null && statut < 400
    ? env.DB.prepare(`INSERT INTO usage (jour, compte, cle, n) VALUES (?, ?, ?, 1)
        ON CONFLICT (jour, compte, cle) DO UPDATE SET n = n + 1`).bind(jour(), compte, actionDe(requete))
    : env.DB.prepare(`INSERT INTO compteurs (jour, cle, n) VALUES (?, ?, 1)
        ON CONFLICT (jour, cle) DO UPDATE SET n = n + 1`)
      .bind(jour(), statut === 404 ? 'introuvable 404' : `${routeDe(requete)} ${statut}`)
  try {
    await ligne.run()
  } catch (erreur) {
    console.error('compteurs non ecrits', erreur)
  }
}

export async function noterActivite(env: Env, compte: string, empreinte: string, requete: Request): Promise<void> {
  const version = versionDe(requete)
  const maintenant = Date.now()
  const avant = maintenant - PAS_ACTIVITE_MS
  try {
    await env.DB.batch([
      env.DB.prepare(`INSERT INTO activite (compte, vu_le, version) VALUES (?, ?, ?)
        ON CONFLICT (compte) DO UPDATE SET vu_le = excluded.vu_le,
          version = COALESCE(excluded.version, activite.version)
        WHERE activite.vu_le < ? OR activite.version IS NOT excluded.version`).bind(compte, maintenant, version, avant),
      env.DB.prepare(`INSERT INTO machines (empreinte_jeton, compte, cree_le, vu_le, version) VALUES (?, ?, ?, ?, ?)
        ON CONFLICT (empreinte_jeton) DO UPDATE SET vu_le = excluded.vu_le,
          version = COALESCE(excluded.version, machines.version)
        WHERE machines.vu_le < ? OR machines.version IS NOT excluded.version`)
        .bind(empreinte, compte, maintenant, maintenant, version, avant),
    ])
  } catch (erreur) {
    console.error('activite non notee', erreur)
  }
}
