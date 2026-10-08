// Responsabilite : le coffre d'un compte — des elements par type (reglages, favoris…), chiffres sur la machine,
// versionnes. Une ecriture precise la version sur laquelle elle s'appuie : si une autre machine a ecrit entre-temps,
// refus 409 avec la version courante, que le client fusionne avant de renvoyer.

import { compteDe, } from './acces'
import { json, lireJson, Refus, texte, type Env } from './outils'

const TYPE = /^[a-z0-9-]{1,32}$/
const MAX_DONNEES = 1_000_000
const MAX_TYPES = 32

export async function lire(requete: Request, env: Env): Promise<Response> {
  const compte = await compteDe(requete, env)
  const lignes = await env.DB.prepare('SELECT type, version, donnees, maj_le FROM coffre WHERE compte = ?').bind(compte)
    .all<{ type: string; version: number; donnees: string; maj_le: number }>()
  return json({ elements: lignes.results })
}

export async function ecrire(requete: Request, env: Env, type: string): Promise<Response> {
  const compte = await compteDe(requete, env)
  if (!TYPE.test(type)) throw new Refus(400, 'type invalide')
  const corps = await lireJson(requete, MAX_DONNEES + 1024)
  const donnees = texte(corps.donnees, 'donnees', MAX_DONNEES)
  const base = Number(corps.base)
  if (!Number.isInteger(base) || base < 0) throw new Refus(400, 'base invalide')
  const actuel = await env.DB.prepare('SELECT version, donnees FROM coffre WHERE compte = ? AND type = ?')
    .bind(compte, type).first<{ version: number; donnees: string }>()
  if ((actuel?.version ?? 0) !== base) return json({ version: actuel?.version ?? 0, donnees: actuel?.donnees ?? null }, 409)
  if (actuel === null) {
    const nombre = await env.DB.prepare('SELECT COUNT(*) AS n FROM coffre WHERE compte = ?').bind(compte)
      .first<{ n: number }>()
    if ((nombre?.n ?? 0) >= MAX_TYPES) throw new Refus(400, 'trop de types')
  }
  const version = base + 1
  // L'ecriture ne passe que si personne n'a ecrit depuis la lecture (version inchangee).
  const resultat = await env.DB.prepare(
    `INSERT INTO coffre (compte, type, version, donnees, maj_le) VALUES (?, ?, ?, ?, ?)
     ON CONFLICT (compte, type) DO UPDATE SET version = excluded.version, donnees = excluded.donnees, maj_le = excluded.maj_le
     WHERE coffre.version = ?`,
  ).bind(compte, type, version, donnees, Date.now(), base).run()
  if (resultat.meta.changes === 0) throw new Refus(409, 'ecriture concurrente, relire')
  return json({ version })
}
