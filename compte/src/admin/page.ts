// Responsabilite : la page du tableau de bord des createurs. Autonome (HTML, style et script en ligne), elle ne
// contient aucune donnee : tout vient de l'API `/v1/admin/...` avec la cle saisie (gardee dans cet onglet seulement).
// Les valeurs recues sont posees en texte (jamais en HTML). Neumorphisme d'Echo, clair et sombre.

const STYLE = `
:root{--shell:#222326;--glow:#2b2d31;--hover:#26282b;--field:#1e1f22;--hair:#303236;--ink:#eaeaeb;--muted:#a5a7ab;
--faint:#71747a;--tint:#9ca3b4;--guard:#55b98d;--warn:#d2a056;--danger:#d5564c;--hi:rgba(255,255,255,.075);
--lo:rgba(0,0,0,.7);color-scheme:dark}
:root[data-theme=light]{--shell:#e1e2e5;--glow:#eeeff1;--hover:#dadbdf;--field:#e6e7ea;--hair:#c8cbd0;--ink:#1f2228;
--muted:#535865;--faint:#858b98;--tint:#586074;--guard:#2f8f66;--warn:#a4712a;--danger:#b8423a;
--hi:rgba(255,255,255,.9);--lo:rgba(82,91,111,.3);color-scheme:light}
*{box-sizing:border-box}
body{margin:0;background:var(--shell);color:var(--ink);font:13px/1.4 'Instrument Sans',system-ui,sans-serif;
-webkit-font-smoothing:antialiased;transition:background .3s}
.num,code{font-family:'JetBrains Mono',ui-monospace,monospace;font-variant-numeric:tabular-nums}
main{max-width:1120px;margin:0 auto;padding:28px 16px 48px;display:flex;flex-direction:column;gap:22px}
header{display:flex;align-items:center;gap:12px}
h1{font-size:17px;font-weight:600;margin:0;flex:1;letter-spacing:-.01em}
h1 small{display:block;font-size:11.5px;font-weight:400;color:var(--faint);margin-top:2px}
h2{font-size:10.5px;font-weight:600;letter-spacing:.12em;text-transform:uppercase;color:var(--faint);margin:0 0 12px}
.carte{background:var(--shell);border-radius:14px;padding:18px;
box-shadow:-6px -6px 16px var(--hi),8px 8px 20px var(--lo)}
.grille{display:grid;gap:18px}
.tuiles{grid-template-columns:repeat(auto-fit,minmax(160px,1fr))}
.deux{grid-template-columns:repeat(auto-fit,minmax(320px,1fr))}
.tuile .valeur{font-size:26px;font-weight:600;letter-spacing:-.02em}
.tuile .libelle{color:var(--muted);font-size:12px}
.tuile .detail{color:var(--faint);font-size:11px;margin-top:4px}
button{font:inherit;color:var(--ink);background:var(--shell);border:0;border-radius:9px;padding:6px 12px;cursor:pointer;
box-shadow:-3px -3px 7px var(--hi),3px 3px 8px var(--lo);display:inline-flex;align-items:center;gap:6px}
button:active{box-shadow:inset -3px -3px 7px var(--hi),inset 3px 3px 8px var(--lo)}
button.danger{color:var(--danger)}
button.icone{padding:7px}
input{font:inherit;color:var(--ink);background:var(--field);border:0;border-radius:9px;padding:8px 12px;outline:0;
box-shadow:inset -2px -2px 5px var(--hi),inset 2px 2px 6px var(--lo);min-width:0}
input::placeholder{color:var(--faint)}
svg{display:block}
.barres{display:flex;align-items:flex-end;gap:3px;height:120px}
.barres div{flex:1;border-radius:3px 3px 0 0;background:var(--tint);min-height:2px;position:relative}
.barres div.err{background:var(--danger)}
.axe{display:flex;justify-content:space-between;color:var(--faint);font-size:10.5px;margin-top:6px}
.ligne{display:flex;align-items:center;gap:10px;padding:5px 0}
.ligne .nom{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.jauge{height:6px;border-radius:3px;background:var(--field);box-shadow:inset 1px 1px 2px var(--lo);flex:1}
.jauge span{display:block;height:100%;border-radius:3px;background:var(--tint)}
table{width:100%;border-collapse:collapse}
th{text-align:left;font-weight:500;color:var(--faint);font-size:11px;padding:6px 8px}
td{padding:8px;border-top:1px solid var(--hair);vertical-align:middle}
td.actions{text-align:right;white-space:nowrap}
.vide{color:var(--faint);font-size:12px;padding:8px 0}
.note{color:var(--faint);font-size:11.5px;text-align:center}
.erreur{color:var(--danger);font-size:12px}
.porte{max-width:380px;margin:12vh auto 0;display:flex;flex-direction:column;gap:12px}
.recherche{display:flex;gap:10px;margin-bottom:10px}
.recherche input{flex:1}
.defile{overflow-x:auto}
[hidden]{display:none!important}
`

const SCRIPT = `
const $ = (s) => document.querySelector(s)
const CLE = 'echo.admin.cle', THEME = 'echo.admin.theme'
const lire = (k) => { try { return sessionStorage.getItem(k) } catch { return null } }
const ecrire = (k, v) => { try { v === null ? sessionStorage.removeItem(k) : sessionStorage.setItem(k, v) } catch {} }
const theme = (() => { try { return localStorage.getItem(THEME) } catch { return null } })()
  ?? (matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark')
document.documentElement.dataset.theme = theme

function el(tag, attrs = {}, ...enfants) {
  const n = document.createElement(tag)
  for (const [k, v] of Object.entries(attrs)) {
    if (k === 'class') n.className = v
    else if (k.startsWith('on')) n.addEventListener(k.slice(2), v)
    else n.setAttribute(k, v)
  }
  for (const e of enfants.flat()) n.append(e instanceof Node ? e : document.createTextNode(String(e)))
  return n
}
const nombre = (n) => new Intl.NumberFormat('fr-FR').format(n ?? 0)
const taille = (o) => o < 1024 ? o + ' o'
  : o < 1048576 ? Math.round(o / 1024) + ' Ko' : (o / 1048576).toFixed(1) + ' Mo'
const date = (ms) => ms ? new Date(ms).toLocaleString('fr-FR', { day: 'numeric', month: 'short', year: '2-digit',
  hour: '2-digit', minute: '2-digit' }) : '—'

async function api(chemin, options = {}) {
  const r = await fetch(chemin, { ...options, headers: { authorization: 'Bearer ' + lire(CLE) } })
  if (r.status === 401 || r.status === 429) { verrouiller((await r.json()).erreur); throw new Error('refus') }
  if (!r.ok && r.status !== 204) throw new Error('erreur ' + r.status)
  return r.status === 204 ? null : r.json()
}

function tuile(libelle, valeur, detail) {
  return el('div', { class: 'carte tuile' }, el('div', { class: 'libelle' }, libelle),
    el('div', { class: 'valeur num' }, valeur), detail ? el('div', { class: 'detail' }, detail) : '')
}

/** Barres par jour sur 30 jours (les jours absents valent 0). */
function barres(lignes, champ, erreurs) {
  const jours = [...Array(30)].map((_, i) => new Date(Date.now() - (29 - i) * 864e5).toISOString().slice(0, 10))
  const par = new Map(lignes.map((l) => [l.jour, l]))
  const max = Math.max(1, ...lignes.map((l) => l[champ]))
  const colonnes = jours.map((j) => {
    const l = par.get(j) ?? {}
    const v = l[champ] ?? 0
    const err = erreurs ? (l[erreurs] ?? 0) : 0
    const titre = j + ' : ' + v + (erreurs ? ' (' + err + ' erreurs)' : '')
    const b = el('div', { title: titre, style: 'height:' + (v / max * 100) + '%' })
    const part = 'position:absolute;bottom:0;left:0;right:0;height:' + (err / v * 100) + '%'
    if (err > 0) b.append(el('div', { class: 'err', style: part }))
    return b
  })
  const axe = el('div', { class: 'axe' }, el('span', {}, jours[0].slice(5)), el('span', {}, "aujourd'hui"))
  return [el('div', { class: 'barres' }, colonnes), axe]
}

function jauges(lignes, nom, valeur, format) {
  if (lignes.length === 0) return el('div', { class: 'vide' }, 'Rien pour l\\'instant.')
  const max = Math.max(1, ...lignes.map(valeur))
  return lignes.map((l) => el('div', { class: 'ligne' }, el('span', { class: 'nom' }, nom(l)),
    el('div', { class: 'jauge' }, el('span', { style: 'width:' + (valeur(l) / max * 100) + '%' })),
    el('span', { class: 'num', style: 'min-width:64px;text-align:right' }, format(valeur(l)))))
}

const NOMS = { reglages: 'Réglages', favoris: 'Favoris', extensions: 'Extensions', onglets: 'Onglets ouverts',
  historique: 'Historique' }

function afficherResume(r) {
  const coffre = r.coffre.reduce((t, c) => t + c.octets, 0)
  $('#tuiles').replaceChildren(
    tuile('Comptes', nombre(r.comptes.n), r.inscriptions.reduce((t, i) => t + i.n, 0) + ' inscrits sur 30 jours'),
    tuile('Actifs', nombre(r.actifs.j7 ?? 0), (r.actifs.j1 ?? 0) + ' sur 24 h · ' + (r.actifs.j30 ?? 0) + ' sur 30 j'),
    tuile('Sessions ouvertes', nombre(r.sessions.n), r.sessions.comptes + ' comptes connectés'),
    tuile('Coffre chiffré', taille(coffre), r.coffre.reduce((t, c) => t + c.n, 0) + ' éléments'),
    tuile('Échecs de connexion', nombre(r.echecsConnexion24h.n), 'sur 24 h'))
  $('#inscriptions').replaceChildren(...barres(r.inscriptions, 'n'))
  $('#requetes').replaceChildren(...barres(r.requetes, 'n', 'erreurs'))
  $('#versions').replaceChildren(...[jauges(r.versions, (v) => 'Echo ' + v.version, (v) => v.n, nombre)].flat())
  $('#types').replaceChildren(...[jauges(r.coffre, (c) => NOMS[c.type] ?? c.type, (c) => c.octets, taille)].flat())
  $('#routes').replaceChildren(...[jauges(r.routes, (x) => x.route, (x) => x.n, nombre)].flat())
}

function actions(c) {
  const cellule = el('td', { class: 'actions' })
  const faire = (chemin, method) => async () => {
    await api('/v1/admin/comptes/' + c.id + chemin, { method }); charger()
  }
  const normal = () => cellule.replaceChildren(
    el('button', { onclick: faire('/deconnexion', 'POST') }, 'Déconnecter'),
    ' ', el('button', { class: 'danger', onclick: confirmer }, 'Supprimer'))
  function confirmer() {
    cellule.replaceChildren(el('span', { class: 'erreur' }, 'Supprimer ' + c.email + ' et tout son coffre ? '),
      el('button', { onclick: normal }, 'Annuler'), ' ',
      el('button', { class: 'danger', onclick: faire('', 'DELETE') }, 'Supprimer définitivement'))
  }
  normal()
  return cellule
}

function afficherComptes(liste) {
  if (liste.length === 0) return $('#comptes').replaceChildren(el('div', { class: 'vide' }, 'Aucun compte.'))
  const lignes = liste.map((c) => el('tr', {}, el('td', {}, c.email), el('td', { class: 'num' }, date(c.creeLe)),
    el('td', { class: 'num' }, date(c.vuLe)), el('td', { class: 'num' }, c.version ?? '—'),
    el('td', { class: 'num' }, taille(c.octets)), el('td', { class: 'num' }, c.sessions), actions(c)))
  $('#comptes').replaceChildren(el('div', { class: 'defile' }, el('table', {}, el('thead', {}, el('tr', {},
    ['E-mail', 'Créé', 'Vu', 'Version', 'Coffre', 'Sessions', ''].map((t) => el('th', {}, t)))),
    el('tbody', {}, lignes))))
}

async function charger() {
  try {
    const recherche = '/v1/admin/comptes?q=' + encodeURIComponent($('#q').value)
    const [r, c] = await Promise.all([api('/v1/admin/resume'), api(recherche)])
    afficherResume(r); afficherComptes(c.comptes)
    const heure = new Date().toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' })
    $('#maj').textContent = 'mis à jour ' + heure
  } catch (e) { if (e.message !== 'refus') $('#maj').textContent = 'lecture impossible : ' + e.message }
}

function verrouiller(message) {
  ecrire(CLE, null); $('#tableau').hidden = true; $('#porte').hidden = false
  $('#message').textContent = message ?? ''
}
function ouvrir() { $('#porte').hidden = true; $('#tableau').hidden = false; charger() }

$('#porte').addEventListener('submit', (e) => {
  e.preventDefault(); ecrire(CLE, $('#cle').value.trim()); $('#cle').value = ''; ouvrir()
})
$('#q').addEventListener('input', () => { clearTimeout(window.attente); window.attente = setTimeout(charger, 250) })
$('#rafraichir').addEventListener('click', charger)
$('#sortir').addEventListener('click', () => verrouiller())
$('#theme').addEventListener('click', () => {
  const t = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark'
  document.documentElement.dataset.theme = t
  try { localStorage.setItem(THEME, t) } catch {}
})
setInterval(() => { if (!$('#tableau').hidden) charger() }, 60000)
lire(CLE) ? ouvrir() : verrouiller()
`

const ICONE_THEME = '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" '
  + 'stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"/>'
  + '<path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>'
  + '</svg>'
const ICONE_RAFRAICHIR = '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" '
  + 'stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 1 1-3-6.7L21 8"/>'
  + '<path d="M21 3v5h-5"/></svg>'

const POLICES = 'https://fonts.googleapis.com/css2?family=Instrument+Sans:wght@400;500;600'
  + '&family=JetBrains+Mono:wght@400;500&display=swap'

export const PAGE_ADMIN = `<!doctype html>
<html lang="fr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="robots" content="noindex"><title>Echo · Tableau de bord</title>
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="${POLICES}">
<style>${STYLE}</style></head><body><main>
<form id="porte" class="porte carte" hidden>
  <h1>Echo<small>Tableau de bord des créateurs</small></h1>
  <input id="cle" type="password" placeholder="Clé d'administration" autocomplete="current-password" required>
  <button type="submit">Entrer</button>
  <div id="message" class="erreur"></div>
</form>
<div id="tableau" hidden style="display:flex;flex-direction:column;gap:22px">
  <header>
    <h1>Echo<small id="maj">chargement…</small></h1>
    <button id="rafraichir" class="icone" title="Rafraîchir" aria-label="Rafraîchir">${ICONE_RAFRAICHIR}</button>
    <button id="theme" class="icone" title="Clair / sombre" aria-label="Clair / sombre">${ICONE_THEME}</button>
    <button id="sortir">Verrouiller</button>
  </header>
  <section id="tuiles" class="grille tuiles"></section>
  <section class="grille deux">
    <div class="carte"><h2>Inscriptions · 30 jours</h2><div id="inscriptions"></div></div>
    <div class="carte">
      <h2>Requêtes · 30 jours <span style="color:var(--danger)">· erreurs</span></h2><div id="requetes"></div>
    </div>
  </section>
  <section class="grille deux">
    <div class="carte"><h2>Versions d'Echo · actifs 30 jours</h2><div id="versions"></div></div>
    <div class="carte"><h2>Coffre par type</h2><div id="types"></div></div>
  </section>
  <section class="carte"><h2>Routes · 7 jours</h2><div id="routes"></div></section>
  <section class="carte"><h2>Comptes</h2>
    <div class="recherche"><input id="q" placeholder="Chercher une adresse e-mail" autocomplete="off"></div>
    <div id="comptes"></div>
  </section>
  <p class="note">
    Les données des utilisateurs sont chiffrées sur leurs machines : ce tableau de bord ne peut pas les lire.
  </p>
</div>
</main><script>${SCRIPT}</script></body></html>`
