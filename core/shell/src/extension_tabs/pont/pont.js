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
    const targets = await chrome.debugger.getTargets()
    reply(targets.filter((t) => t.type === 'page' && t.tabId !== undefined)
      .map((t) => ({ tabId: t.tabId, url: t.url })))
  })
  return true
})
