// Responsabilite : signaux anonymes partages par les navigateurs qui l'ont choisi. Recus par lots (un par installation
// et par jour), sans compte ni identifiant stable, domaines seulement ; ranges en agregats par jour. Rien d'individuel
// n'est garde : le lot lui-meme n'est pas stocke, seulement son jeton du jour (pour refuser un doublon).

import { jour as aujourdhui } from './activite'
import { base64, hmac, json, lireJson, Refus, type Env } from './outils'

const JOUR = /^\d{4}-\d{2}-\d{2}$/
const JETON = /^[0-9a-f]{32}$/
const CLE = /^[a-z0-9.-]{1,253}$/
const VERSION = /^[0-9A-Za-z.+-]{1,32}$/
/** Types acceptes : sites visites, hotes bloques par le bouclier. */
const TYPES = ['sites', 'bloques'] as const
const MAX_CLES = 200
/** Lots acceptes par adresse et par jour (une installation en envoie un, quelques machines derriere une meme box). */
const MAX_PAR_ADRESSE = 3

/** Compte un envoi de cette adresse aujourd'hui ; refuse au-dela de la limite. L'adresse n'est jamais gardee. */
async function limiter(requete: Request, env: Env): Promise<void> {
  const adresse = requete.headers.get('cf-connecting-ip') ?? 'local'
  const jour = aujourdhui()
  const empreinte = base64(await hmac(env.SECRET_SEL, `signaux:${jour}:${adresse}`)).slice(0, 22)
  const ligne = await env.DB.prepare(`INSERT INTO signaux_limite (jour, empreinte, n) VALUES (?, ?, 1)
      ON CONFLICT (jour, empreinte) DO UPDATE SET n = n + 1 RETURNING n`).bind(jour, empreinte).first<{ n: number }>()
  const max = Number(env.MAX_LOTS_ADRESSE ?? MAX_PAR_ADRESSE) || MAX_PAR_ADRESSE
  if ((ligne?.n ?? 0) > max) throw new Refus(429, 'trop de lots aujourd\'hui')
}

function compteurs(valeur: unknown): Array<[string, number]> {
  if (typeof valeur !== 'object' || valeur === null || Array.isArray(valeur)) return []
  return Object.entries(valeur)
    .filter(([cle, n]) => CLE.test(cle) && typeof n === 'number' && Number.isInteger(n) && n > 0)
    .map(([cle, n]): [string, number] => [cle, Math.min(Number(n), 10_000)])
    .slice(0, MAX_CLES)
}

/** Un jour fini et recent (hier jusqu'a 7 jours) : pas de date future, pas de rattrapage ancien. */
function jourAccepte(jour: string): boolean {
  if (!JOUR.test(jour)) return false
  const age = (Date.now() - Date.parse(`${jour}T00:00:00Z`)) / 86_400_000
  return age >= 1 && age < 8
}

export async function recevoir(requete: Request, env: Env): Promise<Response> {
  const corps = await lireJson(requete, 64_000)
  const jour = String(corps.jour ?? '')
  const jeton = String(corps.jeton ?? '')
  if (!jourAccepte(jour) || !JETON.test(jeton)) throw new Refus(400, 'lot invalide')
  await limiter(requete, env)
  const version = typeof corps.version === 'string' && VERSION.test(corps.version) ? corps.version : null
  const nouveau = await env.DB.prepare('INSERT OR IGNORE INTO signaux_lots (jour, jeton, version) VALUES (?, ?, ?)')
    .bind(jour, jeton, version).run()
  if (nouveau.meta.changes === 0) return json({ deja: true })
  const lignes = TYPES.flatMap((type) => compteurs(corps[type]).map(([cle, n]) =>
    env.DB.prepare(`INSERT INTO signaux (jour, type, cle, installs, total) VALUES (?, ?, ?, 1, ?)
      ON CONFLICT (jour, type, cle) DO UPDATE SET installs = installs + 1, total = total + excluded.total`)
      .bind(jour, type, cle, n)))
  if (lignes.length > 0) await env.DB.batch(lignes)
  return json({ recu: lignes.length })
}

const GARDE_JOURS = 90

/** Lecture pour l'administration : seules les cles signalees par au moins `k` installations un meme jour sortent. */
export async function lireSignaux(env: Env): Promise<Response> {
  const k = Math.max(1, Number(env.SEUIL_K ?? 3) || 3)
  const db = env.DB
  const depuis = (jours: number): string => new Date(Date.now() - jours * 86_400_000).toISOString().slice(0, 10)
  await db.batch([
    db.prepare('DELETE FROM signaux WHERE jour < ?').bind(depuis(GARDE_JOURS)),
    db.prepare('DELETE FROM signaux_lots WHERE jour < ?').bind(depuis(GARDE_JOURS)),
    db.prepare('DELETE FROM signaux_limite WHERE jour < ?').bind(depuis(2)),
  ])
  const [lots, versions, sites, bloques] = await db.batch([
    db.prepare('SELECT jour, COUNT(*) AS n FROM signaux_lots WHERE jour >= ? GROUP BY jour ORDER BY jour')
      .bind(depuis(30)),
    db.prepare(`SELECT COALESCE(version, 'inconnue') AS version, COUNT(*) AS n FROM signaux_lots WHERE jour >= ?
      GROUP BY version ORDER BY n DESC`).bind(depuis(30)),
    ...['sites', 'bloques'].map((type) => db.prepare(`SELECT cle, SUM(total) AS total, MAX(installs) AS installs,
        COUNT(*) AS jours FROM signaux WHERE type = ? AND jour >= ? GROUP BY cle HAVING MAX(installs) >= ?
        ORDER BY total DESC LIMIT 25`).bind(type, depuis(30), k)),
  ])
  return json({
    seuil: k, lotsParJour: lots?.results, versions: versions?.results, sites: sites?.results, bloques: bloques?.results,
  })
}
