// Responsabilite : aiguillage du tableau de bord — la page (`/admin`, publique : elle ne contient aucune donnee) et son
// API (`/v1/admin/...`, cle d'administration exigee).

import { json } from '../outils'
import type { Env } from '../outils'
import { changerAdmin, deconnecterCompte, listeComptes, resume, supprimerCompte, verifierAdmin } from './api'
import { ficheCompte } from './fiche'
import { PAGE_ADMIN } from './page'

const ENTETES_PAGE = {
  'content-type': 'text/html; charset=utf-8',
  'content-security-policy': "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline' "
    + "https://fonts.googleapis.com; font-src https://fonts.gstatic.com; connect-src 'self'; img-src data:; "
    + "frame-ancestors 'none'; base-uri 'none'; form-action 'none'",
  'cache-control': 'no-store',
  'referrer-policy': 'no-referrer',
}

/** Une reponse si la requete est pour le tableau de bord, sinon null. */
export async function routeAdmin(requete: Request, env: Env): Promise<Response | null> {
  const { pathname } = new URL(requete.url)
  const methode = requete.method
  if (methode === 'GET' && pathname === '/admin') {
    if (env.ADMIN_KEY === undefined) return json({ erreur: 'introuvable' }, 404)
    return new Response(PAGE_ADMIN, { headers: ENTETES_PAGE })
  }
  if (!pathname.startsWith('/v1/admin/')) return null
  await verifierAdmin(requete, env)
  if (methode === 'GET' && pathname === '/v1/admin/resume') return resume(env)
  if (methode === 'GET' && pathname === '/v1/admin/comptes') return listeComptes(requete, env)
  const [, id, action] = /^\/v1\/admin\/comptes\/([0-9a-f-]{36})(\/deconnexion|\/admin)?$/.exec(pathname) ?? []
  if (id !== undefined && methode === 'POST' && action === '/deconnexion') return deconnecterCompte(env, id)
  if (id !== undefined && methode === 'POST' && action === '/admin') return changerAdmin(requete, env, id)
  if (id !== undefined && methode === 'DELETE' && action === undefined) return supprimerCompte(env, id)
  if (id !== undefined && methode === 'GET' && action === undefined) return ficheCompte(env, id)
  return json({ erreur: 'introuvable' }, 404)
}
