// Responsabilite : produire le paquet de scriptlets et de ressources de remplacement
// que le bouclier injecte dans les pages. Sans lui, les filtres `+js(...)` sont inertes.
//
// Source : uBlock Origin (GPL-3.0), fige sur un commit precis.
// Sortie  : data/shield-resources.json, au format attendu par le moteur de filtrage.

import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const CACHE = join(ROOT, '.cache', 'ublock')
const OUTPUT = join(ROOT, 'data', 'shield-resources.json')
const UBO_REPO = 'https://github.com/gorhill/uBlock.git'
/** Reference uBlock Origin utilisee. La bouger est un choix explicite, pas un effet de bord. */
const UBO_REF = 'master'

function log(step, detail) {
  console.log(`[resources] ${step}${detail ? ` — ${detail}` : ''}`)
}

function syncSource() {
  if (!existsSync(CACHE)) {
    mkdirSync(dirname(CACHE), { recursive: true })
    log('clonage uBlock Origin', UBO_REF)
    execFileSync('git', ['clone', '--depth', '1', '--branch', UBO_REF, UBO_REPO, CACHE], { stdio: 'inherit' })
  } else {
    log('mise a jour de la source')
    execFileSync('git', ['-C', CACHE, 'fetch', '--depth', '1', 'origin', UBO_REF], { stdio: 'inherit' })
    execFileSync('git', ['-C', CACHE, 'reset', '--hard', 'FETCH_HEAD'], { stdio: 'inherit' })
  }
  const commit = execFileSync('git', ['-C', CACHE, 'rev-parse', '--short', 'HEAD']).toString().trim()
  log('source prete', `commit ${commit}`)
  return commit
}

/** Les scriptlets : le code que le bouclier execute dans la page avant les scripts du site. */
async function collectScriptlets() {
  const entry = join(CACHE, 'src', 'js', 'resources', 'scriptlets.js')
  const module = await import(entry)
  const registered = module.builtinScriptlets ?? module.registeredScriptlets
  if (!Array.isArray(registered) || registered.length === 0) {
    throw new Error(`aucun scriptlet extrait de ${entry} — le format amont a change`)
  }
  return registered.map((entry) => ({
    name: entry.name,
    aliases: entry.aliases ?? [],
    kind: { mime: entry.name.endsWith('.fn') ? 'fn/javascript' : 'application/javascript' },
    content: Buffer.from(entry.fn.toString()).toString('base64'),
    dependencies: (entry.dependencies ?? []).filter((d) => typeof d === 'string'),
    ...(entry.requiresTrust ? { permission: 1 } : {}),
  }))
}

/** Lit l'index des ressources redirigeables (`$redirect=`) tel que uBO le declare. */
function parseRedirectIndex() {
  const source = readFileSync(join(CACHE, 'src', 'js', 'redirect-resources.js'), 'utf8')
  const body = source.slice(source.indexOf('['), source.lastIndexOf(']') + 1)
  const entries = []
  const pattern = /\[\s*'([^']+)'\s*,\s*\{([^}]*)\}\s*\]/g
  for (const match of body.matchAll(pattern)) {
    const [, name, props] = match
    const alias = [...props.matchAll(/'([^']+)'/g)].map((m) => m[1]).filter((a) => a !== name)
    entries.push({ name, aliases: alias })
  }
  return entries
}

const MIME_BY_EXTENSION = new Map([
  ['.js', 'application/javascript'], ['.css', 'text/css'], ['.gif', 'image/gif'],
  ['.html', 'text/html'], ['.json', 'application/json'], ['.mp3', 'audio/mp3'],
  ['.mp4', 'video/mp4'], ['.png', 'image/png'], ['.txt', 'text/plain'], ['.xml', 'text/xml'],
])

function mimeOf(name) {
  const dot = name.lastIndexOf('.')
  return MIME_BY_EXTENSION.get(dot < 0 ? '' : name.slice(dot)) ?? 'application/octet-stream'
}

/** Les ressources de remplacement servies a la place d'un script publicitaire bloque. */
function collectRedirects() {
  const dir = join(CACHE, 'src', 'web_accessible_resources')
  const collected = []
  for (const { name, aliases } of parseRedirectIndex()) {
    const path = join(dir, name)
    if (!existsSync(path)) continue
    collected.push({
      name,
      aliases,
      kind: { mime: mimeOf(name) },
      content: readFileSync(path).toString('base64'),
      dependencies: [],
    })
  }
  return collected
}

async function main() {
  const commit = syncSource()
  const scriptlets = await collectScriptlets()
  const redirects = collectRedirects()
  const pack = [...scriptlets, ...redirects]

  const known = new Set(pack.flatMap((r) => [r.name, ...r.aliases]))
  const missing = pack.flatMap((r) => r.dependencies.filter((d) => !known.has(d)))
  if (missing.length > 0) {
    throw new Error(`dependances introuvables dans le paquet : ${[...new Set(missing)].join(', ')}`)
  }

  mkdirSync(dirname(OUTPUT), { recursive: true })
  writeFileSync(OUTPUT, JSON.stringify(pack))
  const size = (JSON.stringify(pack).length / 1024).toFixed(0)
  log('paquet ecrit', `${pack.length} ressources (${scriptlets.length} scriptlets, ${redirects.length} remplacements), ${size} Ko, uBO ${commit}`)
}

main().catch((error) => {
  console.error(`[resources] echec : ${error.message}`)
  process.exit(1)
})
