// Responsabilite : API du tableau de bord des createurs — chiffres d'usage et gestion des comptes. Jamais le contenu du
// coffre (chiffre sur les machines, illisible ici) : seulement comptes, dates, tailles, versions, compteurs.

import { compteDe, effacerCompte } from '../acces'
import { jour } from '../activite'
import { egal, json, lireJson, Refus, type Env } from '../outils'

const JOUR_MS = 24 * 3600 * 1000
const FENETRE_ECHECS_MS = 15 * 60 * 1000
const MAX_ECHECS = 10
const MAX_COMPTES = 200

export async function estAdmin(env: Env, compte: string): Promise<boolean> {
  return (await env.DB.prepare('SELECT 1 FROM admins WHERE compte = ?').bind(compte).first()) !== null
}

/** Session d'un compte administrateur (l'app), ou cle d'administration (la page web, et le premier administrateur).
 * Sans cle configuree et sans session, l'administration n'existe pas (404). */
export async function verifierAdmin(requete: Request, env: Env): Promise<void> {
  const compte = await compteDe(requete, env).catch(() => null)
  if (compte !== null) {
    if (await estAdmin(env, compte)) return
    throw new Refus(403, 'reserve aux administrateurs')
  }
  const cle = env.ADMIN_KEY !== undefined && env.ADMIN_KEY.length >= 24 ? env.ADMIN_KEY : null
  if (cle === null) throw new Refus(404, 'introuvable')
  const depuis = Date.now() - FENETRE_ECHECS_MS
  const echecs = await env.DB.prepare("SELECT COUNT(*) AS n FROM echecs WHERE email = '#admin' AND le > ?").bind(depuis)
    .first<{ n: number }>()
  if ((echecs?.n ?? 0) >= MAX_ECHECS) throw new Refus(429, 'trop de tentatives, reessayer dans un quart d\'heure')
  if (!egal(requete.headers.get('authorization') ?? '', `Bearer ${cle}`)) {
    await env.DB.prepare("INSERT INTO echecs (email, le) VALUES ('#admin', ?)").bind(Date.now()).run()
    throw new Refus(401, 'cle d\'administration incorrecte')
  }
}

/** Les lectures du resume, dans l'ordre ou `resume` les range. */
function lectures(db: D1Database, maintenant: number): D1PreparedStatement[] {
  const il = (jours: number): number => maintenant - jours * JOUR_MS
  return [
    db.prepare('SELECT COUNT(*) AS n FROM comptes'),
    db.prepare(`SELECT date(cree_le / 1000, 'unixepoch') AS jour, COUNT(*) AS n FROM comptes WHERE cree_le > ?
      GROUP BY jour ORDER BY jour`).bind(il(30)),
    db.prepare(`SELECT SUM(vu_le > ?) AS j1, SUM(vu_le > ?) AS j7, SUM(vu_le > ?) AS j30 FROM activite`)
      .bind(il(1), il(7), il(30)),
    db.prepare('SELECT COUNT(*) AS n, COUNT(DISTINCT compte) AS comptes FROM sessions WHERE expire_le > ?')
      .bind(maintenant),
    db.prepare(`SELECT type, COUNT(*) AS n, SUM(length(donnees)) AS octets FROM coffre GROUP BY type
      ORDER BY octets DESC`),
    db.prepare(`SELECT jour, SUM(n) AS n, SUM(CASE WHEN substr(cle, -3) >= '500' THEN n ELSE 0 END) AS erreurs,
      SUM(CASE WHEN substr(cle, -3) BETWEEN '400' AND '499' THEN n ELSE 0 END) AS refus
      FROM compteurs WHERE jour >= ? GROUP BY jour ORDER BY jour`).bind(jour(il(30))),
    db.prepare(`SELECT substr(cle, 1, length(cle) - 4) AS route, SUM(n) AS n FROM compteurs WHERE jour >= ?
      GROUP BY route ORDER BY n DESC LIMIT 12`).bind(jour(il(7))),
    db.prepare(`SELECT COALESCE(version, 'inconnue') AS version, COUNT(*) AS n FROM activite WHERE vu_le > ?
      GROUP BY version ORDER BY n DESC`).bind(il(30)),
    db.prepare("SELECT COUNT(*) AS n FROM echecs WHERE le > ? AND email <> '#admin'").bind(il(1)),
  ]
}

export async function resume(env: Env): Promise<Response> {
  const [comptes, inscriptions, actifs, sessions, coffre, requetes, routes, versions, echecs] =
    await env.DB.batch(lectures(env.DB, Date.now()))
  return json({
    comptes: comptes.results[0], inscriptions: inscriptions.results, actifs: actifs.results[0],
    sessions: sessions.results[0], coffre: coffre.results, requetes: requetes.results, routes: routes.results,
    versions: versions.results, echecsConnexion24h: echecs.results[0],
  })
}

export async function listeComptes(requete: Request, env: Env): Promise<Response> {
  const recherche = (new URL(requete.url).searchParams.get('q') ?? '').trim().toLowerCase().slice(0, 254)
  const motif = `%${recherche.replace(/[\\%_]/g, (c) => `\\${c}`)}%`
  const lignes = await env.DB.prepare(`SELECT c.id, c.email, c.cree_le AS creeLe, a.vu_le AS vuLe, a.version,
      (SELECT COALESCE(SUM(length(donnees)), 0) FROM coffre WHERE compte = c.id) AS octets,
      (SELECT COUNT(*) FROM sessions WHERE compte = c.id AND expire_le > ?) AS sessions,
      EXISTS (SELECT 1 FROM admins WHERE compte = c.id) AS admin
    FROM comptes c LEFT JOIN activite a ON a.compte = c.id WHERE c.email LIKE ? ESCAPE '\\'
    ORDER BY COALESCE(a.vu_le, c.cree_le) DESC LIMIT ?`).bind(Date.now(), motif, MAX_COMPTES).all()
  return json({ comptes: lignes.results })
}

export async function deconnecterCompte(env: Env, id: string): Promise<Response> {
  const fait = await env.DB.prepare('DELETE FROM sessions WHERE compte = ?').bind(id).run()
  return json({ sessionsFermees: fait.meta.changes })
}

/** Donne ou retire l'acces administrateur a un compte. */
export async function changerAdmin(requete: Request, env: Env, id: string): Promise<Response> {
  const admin = (await lireJson(requete)).admin === true
  const existe = await env.DB.prepare('SELECT 1 FROM comptes WHERE id = ?').bind(id).first()
  if (existe === null) throw new Refus(404, 'compte introuvable')
  await env.DB.prepare(admin ? 'INSERT OR IGNORE INTO admins (compte) VALUES (?)' : 'DELETE FROM admins WHERE compte = ?')
    .bind(id).run()
  return json({ admin })
}

export async function supprimerCompte(env: Env, id: string): Promise<Response> {
  const existe = await env.DB.prepare('SELECT 1 FROM comptes WHERE id = ?').bind(id).first()
  if (existe === null) throw new Refus(404, 'compte introuvable')
  await effacerCompte(env, id)
  return new Response(null, { status: 204 })
}
