// Responsabilite : la fiche d'un compte pour l'administration — usage par jour et par action (30 jours), machines
// connectees (dates, version), coffre par type (version, taille chiffree, date). Jamais le contenu.

import { jour } from '../activite'
import { json, Refus, type Env } from '../outils'

const JOUR_MS = 24 * 3600 * 1000

export async function ficheCompte(env: Env, id: string): Promise<Response> {
  const db = env.DB
  const depuis = jour(Date.now() - 30 * JOUR_MS)
  const [compte, parJour, parAction, machines, coffre] = await db.batch([
    db.prepare(`SELECT c.id, c.email, c.cree_le AS creeLe, a.vu_le AS vuLe, a.version,
      EXISTS (SELECT 1 FROM admins WHERE compte = c.id) AS admin
      FROM comptes c LEFT JOIN activite a ON a.compte = c.id WHERE c.id = ?`).bind(id),
    db.prepare('SELECT jour, SUM(n) AS n FROM usage WHERE compte = ? AND jour >= ? GROUP BY jour ORDER BY jour')
      .bind(id, depuis),
    db.prepare(`SELECT cle AS action, SUM(n) AS n FROM usage WHERE compte = ? AND jour >= ? GROUP BY cle
      ORDER BY n DESC`)
      .bind(id, depuis),
    db.prepare(`SELECT m.cree_le AS creeLe, m.vu_le AS vuLe, m.version, s.expire_le AS expireLe FROM machines m
      LEFT JOIN sessions s ON s.empreinte_jeton = m.empreinte_jeton WHERE m.compte = ? ORDER BY m.vu_le DESC`).bind(id),
    db.prepare(`SELECT type, version, length(donnees) AS octets, maj_le AS majLe FROM coffre WHERE compte = ?
      ORDER BY octets DESC`).bind(id),
  ])
  const infos = compte?.results[0]
  if (infos === undefined) throw new Refus(404, 'compte introuvable')
  return json({
    compte: infos, parJour: parJour?.results, parAction: parAction?.results, machines: machines?.results,
    coffre: coffre?.results,
  })
}
