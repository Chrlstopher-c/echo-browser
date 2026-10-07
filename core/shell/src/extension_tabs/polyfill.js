// Injecte par Echo dans les pages d'extension. `chrome.tabs.query` y renvoie les onglets d'Echo (actif compris) avec
// leur vrai identifiant, obtenu du pont interne ; sans reponse du pont, la requete d'origine est rendue telle quelle.
(() => {
  window.__echoTabs = __DATA__
  if (window.__echoTabsPatched || !self.chrome || !chrome.tabs || !chrome.runtime) return
  window.__echoTabsPatched = true
  const PONT = '__PONT__'
  const query = chrome.tabs.query.bind(chrome.tabs)
  const glob = (p) => new RegExp('^' + p.replace(/[.+?^${}()|[\]\\]/g, '\\$&').replace(/\*/g, '.*') + '$')
  const hostMatches = (pattern, host) =>
    pattern === '*' || host === pattern || (pattern.startsWith('*.') &&
      (host === pattern.slice(2) || host.endsWith(pattern.slice(1))))
  const urlMatches = (pattern, url) => {
    if (pattern === '<all_urls>') return /^(https?|wss?|file|ftp):/.test(url)
    const m = /^(\*|https?|wss?|file|ftp):\/\/([^/]*)(\/.*)$/.exec(pattern)
    let u
    try { u = new URL(url) } catch { return false }
    if (!m) return false
    const scheme = m[1] === '*' ? /^https?:$/.test(u.protocol) : u.protocol === m[1] + ':'
    return scheme && hostMatches(m[2], u.hostname) && glob(m[3]).test(u.pathname + u.search)
  }
  const keep = (tab, info) => Object.entries(info).every(([k, v]) => {
    if (v === undefined) return true
    if (k === 'url') return [].concat(v).some((p) => urlMatches(p, tab.url || ''))
    if (k === 'title') return glob(v).test(tab.title || '')
    if (k === 'currentWindow' || k === 'lastFocusedWindow') return v === true
    if (k === 'windowType') return v === 'normal'
    if (k === 'windowId') return v === tab.windowId || v === chrome.windows?.WINDOW_ID_CURRENT
    return !(k in tab) || tab[k] === v
  })
  const targets = () => new Promise((done) => chrome.runtime.sendMessage(PONT, 'echo:onglets', (r) => {
    void chrome.runtime.lastError
    done(Array.isArray(r) ? r : [])
  }))
  const echoTabs = async () => {
    const free = await targets()
    if (free.length === 0) return []
    const win = (await query({}))[0]?.windowId ?? -1
    const out = []
    for (const t of window.__echoTabs.tabs) {
      const at = free.findIndex((x) => x.url === t.url)
      if (at < 0) continue
      const [{ tabId }] = free.splice(at, 1)
      try {
        const tab = await chrome.tabs.get(tabId)
        out.push({ ...tab, windowId: win, index: out.length, active: t.active, highlighted: t.active,
          selected: t.active, pinned: t.pinned })
      } catch {}
    }
    return out
  }
  chrome.tabs.query = function (info = {}, callback) {
    const run = echoTabs().then((tabs) => (tabs.length > 0 ? tabs.filter((t) => keep(t, info)) : query(info)))
    if (typeof callback !== 'function') return run
    run.then(callback, () => callback([]))
  }
})()
