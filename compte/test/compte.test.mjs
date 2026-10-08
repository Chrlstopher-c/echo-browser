// Tests du service de compte contre une instance locale (`wrangler dev`, base D1 locale) : COMPTE_URL.
import assert from 'node:assert/strict'

const URL_SERVICE = process.env.COMPTE_URL ?? 'http://127.0.0.1:8788'
const b64 = (o) => Buffer.from(o).toString('base64')
const cle = () => b64(crypto.getRandomValues(new Uint8Array(32)))
const email = (n) => `essai-${n}-${Date.now()}@exemple.org`

async function appel(methode, chemin, corps, jeton) {
  const reponse = await fetch(URL_SERVICE + chemin, {
    method: methode,
    headers: { 'content-type': 'application/json', ...(jeton ? { authorization: `Bearer ${jeton}` } : {}) },
    body: corps === undefined ? undefined : JSON.stringify(corps),
  })
  const texte = await reponse.text()
  return { statut: reponse.status, corps: texte ? JSON.parse(texte) : null }
}

const essais = []
const essai = (nom, f) => essais.push([nom, f])

essai('sel fictif stable pour une adresse inconnue', async () => {
  const e = email('inconnu')
  const a = await appel('POST', '/v1/sel', { email: e })
  const b = await appel('POST', '/v1/sel', { email: e })
  assert.equal(a.statut, 200)
  assert.equal(a.corps.sel, b.corps.sel)
  assert.equal(Buffer.from(a.corps.sel, 'base64').length, 16)
})

essai('inscription, doublon refuse, connexion, mauvaise cle refusee', async () => {
  const e = email('compte'); const k = cle(); const sel = b64(crypto.getRandomValues(new Uint8Array(16)))
  const ins = await appel('POST', '/v1/inscription', { email: e, cleAcces: k, sel, iterations: 600000 })
  assert.equal(ins.statut, 201); assert.ok(ins.corps.jeton)
  assert.equal((await appel('POST', '/v1/inscription', { email: e, cleAcces: k, sel, iterations: 600000 })).statut, 409)
  assert.equal((await appel('POST', '/v1/sel', { email: e })).corps.sel, sel)
  assert.equal((await appel('POST', '/v1/connexion', { email: e.toUpperCase(), cleAcces: k })).statut, 200)
  assert.equal((await appel('POST', '/v1/connexion', { email: e, cleAcces: cle() })).statut, 401)
})

essai('trop de tentatives : 429', async () => {
  const e = email('force'); const sel = b64(crypto.getRandomValues(new Uint8Array(16)))
  await appel('POST', '/v1/inscription', { email: e, cleAcces: cle(), sel, iterations: 600000 })
  for (let i = 0; i < 10; i++) await appel('POST', '/v1/connexion', { email: e, cleAcces: cle() })
  assert.equal((await appel('POST', '/v1/connexion', { email: e, cleAcces: cle() })).statut, 429)
})

essai('coffre : versions, conflit 409, session requise, deconnexion, suppression', async () => {
  const e = email('coffre'); const k = cle(); const sel = b64(crypto.getRandomValues(new Uint8Array(16)))
  const { jeton } = (await appel('POST', '/v1/inscription', { email: e, cleAcces: k, sel, iterations: 600000 })).corps
  assert.deepEqual((await appel('GET', '/v1/coffre', undefined, jeton)).corps.elements, [])
  assert.equal((await appel('PUT', '/v1/coffre/favoris', { base: 0, donnees: 'AAA' }, jeton)).corps.version, 1)
  const conflit = await appel('PUT', '/v1/coffre/favoris', { base: 0, donnees: 'BBB' }, jeton)
  assert.equal(conflit.statut, 409); assert.equal(conflit.corps.version, 1); assert.equal(conflit.corps.donnees, 'AAA')
  assert.equal((await appel('PUT', '/v1/coffre/favoris', { base: 1, donnees: 'CCC' }, jeton)).corps.version, 2)
  const lu = (await appel('GET', '/v1/coffre', undefined, jeton)).corps.elements
  assert.equal(lu.length, 1); assert.equal(lu[0].donnees, 'CCC'); assert.equal(lu[0].version, 2)
  assert.equal((await appel('GET', '/v1/coffre')).statut, 401)
  assert.equal((await appel('PUT', '/v1/coffre/../x', { base: 0, donnees: 'x' }, jeton)).statut, 404)
  const autre = (await appel('POST', '/v1/connexion', { email: e, cleAcces: k })).corps.jeton
  assert.equal((await appel('POST', '/v1/deconnexion', undefined, jeton)).statut, 204)
  assert.equal((await appel('GET', '/v1/coffre', undefined, jeton)).statut, 401)
  assert.equal((await appel('DELETE', '/v1/compte', undefined, autre)).statut, 204)
  assert.equal((await appel('POST', '/v1/connexion', { email: e, cleAcces: k })).statut, 401)
})

const CLE_ADMIN = process.env.ADMIN_KEY
async function admin(methode, chemin, cleAdmin = CLE_ADMIN) {
  const r = await fetch(URL_SERVICE + chemin, { method: methode, headers: { authorization: `Bearer ${cleAdmin}` } })
  const texte = await r.text()
  return { statut: r.status, corps: texte ? JSON.parse(texte) : null }
}

if (CLE_ADMIN) essai('tableau de bord : cle exigee, chiffres, version, deconnexion et suppression', async () => {
  assert.equal((await admin('GET', '/v1/admin/resume', 'mauvaise-cle')).statut, 401)
  const page = await fetch(URL_SERVICE + '/admin')
  assert.equal(page.status, 200); assert.match(await page.text(), /Tableau de bord/)
  const e = email('admin'); const k = cle(); const sel = b64(crypto.getRandomValues(new Uint8Array(16)))
  const { jeton } = (await appel('POST', '/v1/inscription', { email: e, cleAcces: k, sel, iterations: 600000 })).corps
  await fetch(URL_SERVICE + '/v1/coffre', { headers: { authorization: `Bearer ${jeton}`, 'x-echo-version': '9.9.9' } })
  await appel('PUT', '/v1/coffre/favoris', { base: 0, donnees: 'DONNEES-CHIFFREES' }, jeton)
  const resume = (await admin('GET', '/v1/admin/resume')).corps
  assert.ok(resume.comptes.n >= 1); assert.ok(resume.sessions.n >= 1)
  assert.ok(resume.versions.some((v) => v.version === '9.9.9'))
  assert.ok(resume.requetes.some((r) => r.n > 0))
  const [compte] = (await admin('GET', `/v1/admin/comptes?q=${encodeURIComponent(e)}`)).corps.comptes
  assert.equal(compte.email, e); assert.equal(compte.version, '9.9.9'); assert.ok(compte.octets > 0)
  assert.ok(!JSON.stringify(resume).includes('DONNEES-CHIFFREES'), 'le resume ne contient jamais le coffre')
  await new Promise((r) => setTimeout(r, 300))
  const fiche = (await admin('GET', `/v1/admin/comptes/${compte.id}`)).corps
  assert.equal(fiche.compte.email, e)
  const actions = fiche.parAction.map((x) => x.action)
  assert.ok(actions.includes('synchro') && actions.includes('ecriture:favoris') && actions.includes('inscription'), actions)
  assert.equal(fiche.machines.length, 1); assert.equal(fiche.machines[0].version, '9.9.9')
  assert.equal(fiche.coffre[0].type, 'favoris'); assert.ok(!JSON.stringify(fiche).includes('DONNEES-CHIFFREES'))
  const usage = (await admin('GET', '/v1/admin/resume')).corps
  assert.ok(usage.actifsParJour.length >= 1 && usage.plusActifs.some((p) => p.email === e), 'usage global')
  assert.equal((await appel('GET', '/v1/moi', undefined, jeton)).corps.admin, false)
  assert.equal((await appel('GET', '/v1/admin/resume', undefined, jeton)).statut, 403)
  const rendre = await fetch(`${URL_SERVICE}/v1/admin/comptes/${compte.id}/admin`, { method: 'POST',
    headers: { authorization: `Bearer ${CLE_ADMIN}` }, body: JSON.stringify({ admin: true }) })
  assert.equal(rendre.status, 200)
  assert.equal((await appel('GET', '/v1/moi', undefined, jeton)).corps.admin, true)
  assert.equal((await appel('GET', '/v1/admin/resume', undefined, jeton)).statut, 200, 'session admin acceptee')
  assert.equal((await admin('POST', `/v1/admin/comptes/${compte.id}/deconnexion`)).corps.sessionsFermees, 1)
  assert.equal((await appel('GET', '/v1/coffre', undefined, jeton)).statut, 401)
  assert.equal((await admin('DELETE', `/v1/admin/comptes/${compte.id}`)).statut, 204)
  assert.equal((await appel('POST', '/v1/connexion', { email: e, cleAcces: k })).statut, 401)
  assert.equal((await admin('GET', `/v1/admin/comptes?q=${encodeURIComponent(e)}`)).corps.comptes.length, 0)
})

let echecs = 0
for (const [nom, f] of essais) {
  try { await f(); console.log(`ok  ${nom}`) } catch (e) { echecs++; console.log(`ECHEC ${nom} : ${e.message}`) }
}
process.exit(echecs === 0 ? 0 : 1)
