// Responsabilite : point d'entree du service de compte Echo — aiguillage des routes et conversion des refus.

import { connexion, deconnexion, inscription, sel, supprimer } from './acces'
import { ecrire, lire } from './coffre'
import { json, Refus, type Env } from './outils'

async function router(requete: Request, env: Env): Promise<Response> {
  const { pathname } = new URL(requete.url)
  const methode = requete.method
  if (methode === 'GET' && pathname === '/v1/sante') return json({ ok: true })
  if (methode === 'POST' && pathname === '/v1/sel') return sel(requete, env)
  if (methode === 'POST' && pathname === '/v1/inscription') return inscription(requete, env)
  if (methode === 'POST' && pathname === '/v1/connexion') return connexion(requete, env)
  if (methode === 'POST' && pathname === '/v1/deconnexion') return deconnexion(requete, env)
  if (methode === 'DELETE' && pathname === '/v1/compte') return supprimer(requete, env)
  if (methode === 'GET' && pathname === '/v1/coffre') return lire(requete, env)
  const type = /^\/v1\/coffre\/([^/]+)$/.exec(pathname)?.[1]
  if (methode === 'PUT' && type !== undefined) return ecrire(requete, env, type)
  return json({ erreur: 'introuvable' }, 404)
}

export default {
  async fetch(requete: Request, env: Env): Promise<Response> {
    try {
      return await router(requete, env)
    } catch (erreur) {
      if (erreur instanceof Refus) return json({ erreur: erreur.message }, erreur.statut)
      console.error(erreur)
      return json({ erreur: 'erreur interne' }, 500)
    }
  },
}
