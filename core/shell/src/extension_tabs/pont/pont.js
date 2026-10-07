// Interne a Echo. Les onglets d'Echo ne sont dans aucune fenetre Chrome : `tabs.query` ne les voit pas, mais
// `debugger.getTargets` donne leur identifiant d'onglet. Repond seulement aux extensions qui voient deja les onglets.
const BROAD = ['<all_urls>', '*://*/*', 'http://*/*', 'https://*/*']

async function allowed(id) {
  try {
    const info = await chrome.management.get(id)
    return info.permissions.includes('tabs') || info.hostPermissions.some((p) => BROAD.includes(p))
  } catch {
    return false
  }
}

chrome.runtime.onMessageExternal.addListener((message, sender, reply) => {
  if (message !== 'echo:onglets') return false
  allowed(sender.id).then(async (ok) => {
    if (!ok) return reply([])
    const targets = (await chrome.debugger.getTargets()).filter((t) => t.type === 'page' && t.tabId !== undefined)
    // `getTargets` voit tous les profils ; `tabs.get` echoue pour un onglet d'un autre profil : on ne rend que les siens.
    const mine = await Promise.all(targets.map((t) => chrome.tabs.get(t.tabId).then(() => t, () => null)))
    reply(mine.filter(Boolean).map((t) => ({ tabId: t.tabId, url: t.url })))
  })
  return true
})

importScripts('regle.js')

// Au demarrage du profil et a chaque extension que Chromium y installe. Les changements faits pendant la session
// passent par `appliquer.html` : un cookie pose par Echo ne declenche pas `cookies.onChanged`.
chrome.runtime.onStartup.addListener(enforce)
chrome.runtime.onInstalled.addListener(enforce)
chrome.management.onInstalled.addListener(enforce)
