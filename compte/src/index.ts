// Responsabilite : point d'entree du service de compte Echo — aiguillage des routes et conversion des refus.

import { connexion, deconnexion, etat, inscription, moi, sel, supprimer } from './acces'
import { ecrire, lire } from './coffre'
import { recevoir } from './signaux'
import { compter } from './activite'
import { routeAdmin } from './admin/routes'
import { attendre, json, lier, Refus, type Env } from './outils'

async function router(requete: Request, env: Env): Promise<Response> {
  const { pathname } = new URL(requete.url)
  const methode = requete.method
  if (methode === 'GET' && pathname === '/v1/sante') return json({ ok: true })
  const admin = await routeAdmin(requete, env)
  if (admin !== null) return admin
  if (methode === 'POST' && pathname === '/v1/sel') return sel(requete, env)
  if (methode === 'POST' && pathname === '/v1/inscription') return inscription(requete, env)
  if (methode === 'POST' && pathname === '/v1/connexion') return connexion(requete, env)
  if (methode === 'POST' && pathname === '/v1/deconnexion') return deconnexion(requete, env)
  if (methode === 'DELETE' && pathname === '/v1/compte') return supprimer(requete, env)
  if (methode === 'GET' && pathname === '/v1/coffre') return lire(requete, env)
  if (methode === 'GET' && pathname === '/v1/moi') return moi(requete, env)
  if (methode === 'GET' && pathname === '/v1/etat') return etat(requete, env)
  if (methode === 'POST' && pathname === '/v1/signaux') return recevoir(requete, env)
  const type = /^\/v1\/coffre\/([^/]+)$/.exec(pathname)?.[1]
  if (methode === 'PUT' && type !== undefined) return ecrire(requete, env, type)
  return json({ erreur: 'introuvable' }, 404)
}

async function repondre(requete: Request, env: Env): Promise<Response> {
  try {
    return await router(requete, env)
  } catch (erreur) {
    if (erreur instanceof Refus) return json({ erreur: erreur.message }, erreur.statut)
    console.error(erreur)
    return json({ erreur: 'erreur interne' }, 500)
  }
}

export default {
  async fetch(requete: Request, env: Env, contexte: ExecutionContext): Promise<Response> {
    lier(requete, contexte)
    const reponse = await repondre(requete, env)
    if (new URL(requete.url).pathname !== '/v1/etat') attendre(requete, compter(env, requete, reponse.status))
    return reponse
  },
}
