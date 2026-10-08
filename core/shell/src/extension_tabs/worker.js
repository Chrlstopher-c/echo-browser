// Injecte par Echo dans le service worker d'une extension, avant son premier script (pause du debogueur). Les onglets
// d'Echo ne sont dans aucune fenetre Chrome : ni `tabs.query` ni les evenements d'onglets ne les voient. Ici, Echo
// pousse sa liste (`__echoTabs`) ; `tabs.query` la rend avec les vrais identifiants (pont interne) et les ecouteurs
// `onCreated/onUpdated/onRemoved/onActivated` recoivent les changements, y compris ceux faits pendant la veille.
(() => {
  if (self.__echoTabsReady || !self.chrome || !chrome.tabs || !chrome.runtime) return
  self.__echoTabsReady = true
  const PONT = '__PONT__'
  let state = null
  let haveState
  const firstState = new Promise((resolve) => { haveState = resolve })
  const ids = new Map()
  const listeners = { onCreated: [], onUpdated: [], onRemoved: [], onActivated: [] }
  for (const name of Object.keys(listeners)) {
    const event = chrome.tabs[name]
    const add = event.addListener.bind(event)
    const remove = event.removeListener.bind(event)
    event.addListener = (fn, ...rest) => { listeners[name].push(fn); return add(fn, ...rest) }
    event.removeListener = (fn) => { listeners[name] = listeners[name].filter((g) => g !== fn); return remove(fn) }
  }
  const query = chrome.tabs.query.bind(chrome.tabs)
  const fire = (name, ...args) => listeners[name].forEach((fn) => { try { fn(...args) } catch (e) { console.error(e) } })
  const targets = () => new Promise((done) => chrome.runtime.sendMessage(PONT, 'echo:onglets', (r) => {
    void chrome.runtime.lastError
    done(Array.isArray(r) ? r : [])
  }))
  let windowId
  const win = async () => (windowId ??= (await query({}))[0]?.windowId ?? -1)

  async function resolveIds() {
    if (state.tabs.every((t) => ids.has(t.e))) return
    const taken = new Set(ids.values())
    const free = (await targets()).filter((t) => !taken.has(t.tabId))
    for (const t of state.tabs) {
      if (ids.has(t.e)) continue
      const at = free.findIndex((x) => x.url === t.url)
      if (at >= 0) ids.set(t.e, free.splice(at, 1)[0].tabId)
    }
  }

  async function echoTabs() {
    await firstState
    await resolveIds()
    const w = await win()
    const out = []
    for (const t of state.tabs) {
      const id = ids.get(t.e)
      if (id === undefined) continue
      try {
        const tab = await chrome.tabs.get(id)
        out.push({ ...tab, url: t.url, title: t.title, status: t.status, windowId: w, index: out.length,
          active: t.active, highlighted: t.active, selected: t.active, pinned: t.pinned })
      } catch {}
    }
    return out
  }

  const glob = (p) => new RegExp('^' + p.replace(/[.+?^${}()|[\]\\]/g, '\\$&').replace(/\*/g, '.*') + '$')
  const urlMatches = (pattern, url) => {
    if (pattern === '<all_urls>') return /^(https?|wss?|file|ftp):/.test(url)
    const m = /^(\*|https?|wss?|file|ftp):\/\/([^/]*)(\/.*)$/.exec(pattern)
    let u
    try { u = new URL(url) } catch { return false }
    if (!m) return false
    const scheme = m[1] === '*' ? /^https?:$/.test(u.protocol) : u.protocol === m[1] + ':'
    const host = m[2] === '*' || u.hostname === m[2] ||
      (m[2].startsWith('*.') && (u.hostname === m[2].slice(2) || u.hostname.endsWith(m[2].slice(1))))
    return scheme && host && glob(m[3]).test(u.pathname + u.search)
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
  chrome.tabs.query = function (info = {}, callback) {
    const run = echoTabs().then((tabs) => (tabs.length > 0 ? tabs.filter((t) => keep(t, info)) : query(info)))
    if (typeof callback !== 'function') return run
    run.then(callback, () => callback([]))
  }

  async function dispatch(previous, current) {
    await resolveIds()
    const w = await win()
    const objects = await echoTabs()
    const objectOf = (e) => objects.find((o) => o.id === ids.get(e))
    const before = new Map(previous.tabs.map((t) => [t.e, t]))
    for (const t of current.tabs) {
      const o = objectOf(t.e)
      const p = before.get(t.e)
      if (!o) continue
      if (!p) {
        fire('onCreated', o)
      } else {
        const change = {}
        for (const key of ['url', 'status', 'title', 'pinned']) if (p[key] !== t[key]) change[key] = t[key]
        if (Object.keys(change).length > 0) fire('onUpdated', o.id, change, o)
      }
      if (t.active && !(p && p.active)) fire('onActivated', { tabId: o.id, windowId: w })
    }
    const after = new Set(current.tabs.map((t) => t.e))
    for (const p of previous.tabs) {
      const id = ids.get(p.e)
      if (after.has(p.e) || id === undefined) continue
      ids.delete(p.e)
      fire('onRemoved', id, { windowId: w, isWindowClosing: false })
    }
  }

  // Au reveil, Echo envoie d'abord l'etat vu avant la veille, puis l'actuel : les changements faits pendant la veille
  // partent apres le premier script (setTimeout), une fois les ecouteurs enregistres.
  self.__echoTabs = (next, asleep) => {
    const previous = asleep ?? state
    state = next
    haveState()
    if (previous !== null) setTimeout(() => dispatch(previous, next).catch((e) => console.error(e)), 0)
  }
  self.__echoListening = () => Object.values(listeners).some((l) => l.length > 0)
})()
