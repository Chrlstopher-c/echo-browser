// Un profil d'Echo est une identite : ses extensions ne se partagent pas. Chromium installe une extension declaree
// dans tous ses profils ; ici, dans chacun, on ne garde actives que celles qu'Echo attribue a ce profil. Echo pose dans
// chaque profil le cookie `extensions` (identifiants separes par des virgules) ; sans lui, on ne touche a rien.
// Partage par le service worker (`pont.js`) et par la page `appliquer.html`, qu'Echo ouvre apres un changement.
const MARK = { url: 'https://echo-profil.invalid/', name: 'extensions' }
const UNTOUCHED = ['development', 'admin']

async function enforce() {
  const mark = await chrome.cookies.get(MARK)
  if (!mark) return
  const allowed = mark.value.split(',').filter(Boolean)
  for (const extension of await chrome.management.getAll()) {
    if (extension.id === chrome.runtime.id || extension.type !== 'extension') continue
    if (UNTOUCHED.includes(extension.installType)) continue
    const wanted = allowed.includes(extension.id)
    if (extension.enabled !== wanted) await chrome.management.setEnabled(extension.id, wanted).catch(() => {})
  }
}
