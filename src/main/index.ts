import { app, BrowserWindow, ipcMain, WebContentsView, session, Menu, clipboard, shell } from 'electron'
import { join } from 'path'
import { createAdBlocker } from './adBlocker'
import { loadSettings, saveSettings } from './settings'
import {
  isBookmarked, toggleBookmark, deleteBookmark, getBookmarks,
  addHistory, getHistory, deleteHistory, clearHistory,
  getStoredDownloads,
} from './store'
import { setupDownloads, cancelDownload, openDownload, showDownloadInFolder, getActiveDownloads } from './downloads'
import type { ElectronBlocker } from '@cliqz/adblocker-electron'
import type { TabInfo } from '../shared/types'
import type { Settings } from './settings'

const HEADER_HEIGHT = 116
const FIND_BAR_HEIGHT = 48
const CHROME_UA = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36'
const BROWSER_PARTITION = 'persist:browser'

let win: BrowserWindow
let blocker: ElectronBlocker
let browserSession: Electron.Session
let adBlockerEnabled = true
let activeTabId = 0
let nextId = 1
let settings: Settings
let contentFullscreen = false
let findBarOpen = false

interface Tab extends TabInfo { view: WebContentsView }
const tabs: Tab[] = []

function getActive(): Tab | undefined { return tabs.find(t => t.id === activeTabId) }
function sameDomain(a: string, b: string): boolean {
  try { return new URL(a).hostname === new URL(b).hostname } catch { return false }
}
function getHostname(url: string): string {
  try { return new URL(url).hostname } catch { return url }
}

function pushState(): void {
  const active = getActive()
  win.webContents.send('tabs-update', {
    tabs: tabs.map(({ id, title, url, favicon, loading, canGoBack, canGoForward, zoom }) =>
      ({ id, title, url, favicon, loading, canGoBack, canGoForward, zoom })),
    activeTabId,
    adBlockerEnabled,
    isBookmarked: active ? isBookmarked(active.url) : false,
  })
}

function pushSettings(): void { win.webContents.send('settings-update', settings) }

function hideActiveView(): void {
  const t = tabs.find(tab => tab.id === activeTabId)
  if (t) t.view.setBounds({ x: 0, y: 0, width: 0, height: 0 })
}

function updateBounds(w?: number, h?: number): void {
  const [width, height] = w && h ? [w, h] : win.getContentSize()
  const headerH = HEADER_HEIGHT + (findBarOpen && !contentFullscreen ? FIND_BAR_HEIGHT : 0)
  const y = contentFullscreen ? 0 : headerH
  const ch = contentFullscreen ? height : height - headerH
  for (const t of tabs) {
    t.view.setBounds(t.id === activeTabId
      ? { x: 0, y, width, height: Math.max(0, ch) }
      : { x: 0, y: 0, width: 0, height: 0 })
  }
}

function setZoom(tab: Tab, factor: number): void {
  tab.zoom = Math.round(Math.max(0.3, Math.min(3.0, factor)) * 10) / 10
  tab.view.webContents.setZoomFactor(tab.zoom)
  pushState()
}

function closeFindBar(): void {
  if (!findBarOpen) return
  findBarOpen = false
  getActive()?.view.webContents.stopFindInPage('clearSelection')
  updateBounds()
  win.webContents.send('find-force-close')
}

function setupTabShortcuts(view: WebContentsView, tab: Tab): void {
  view.webContents.on('before-input-event', (event, input) => {
    if (input.type !== 'keyDown') return
    const ctrl = input.control || input.meta
    const shift = input.shift
    const alt = input.alt

    if (ctrl && input.key === 't') { event.preventDefault(); const t = createTab(); switchTab(t.id); return }
    if (ctrl && input.key === 'w') { event.preventDefault(); closeTab(tab.id); return }
    if (ctrl && !shift && input.key === 'Tab') {
      event.preventDefault()
      const idx = tabs.findIndex(t => t.id === activeTabId)
      switchTab(tabs[(idx + 1) % tabs.length].id); return
    }
    if (ctrl && shift && input.key === 'Tab') {
      event.preventDefault()
      const idx = tabs.findIndex(t => t.id === activeTabId)
      switchTab(tabs[(idx - 1 + tabs.length) % tabs.length].id); return
    }
    if (ctrl && input.key === 'r') { event.preventDefault(); view.webContents.reload(); return }
    if (ctrl && input.key === 'l') { event.preventDefault(); win.webContents.send('focus-url-bar'); return }
    if (alt && input.key === 'ArrowLeft') { event.preventDefault(); view.webContents.navigationHistory.goBack(); return }
    if (alt && input.key === 'ArrowRight') { event.preventDefault(); view.webContents.navigationHistory.goForward(); return }
    if (ctrl && (input.key === '+' || input.key === '=')) { event.preventDefault(); setZoom(tab, tab.zoom + 0.1); return }
    if (ctrl && input.key === '-') { event.preventDefault(); setZoom(tab, tab.zoom - 0.1); return }
    if (ctrl && input.key === '0') { event.preventDefault(); setZoom(tab, 1.0); return }
    if (ctrl && input.key === 'f') {
      event.preventDefault(); findBarOpen = true; updateBounds(); win.webContents.send('find-open'); return
    }
    if (input.key === 'Escape' && findBarOpen) { closeFindBar(); return }
    if (ctrl && shift && input.key === 'I') { event.preventDefault(); view.webContents.openDevTools(); return }
    if (ctrl && input.key === 'd') {
      event.preventDefault()
      toggleBookmark({ url: tab.url, title: tab.title, favicon: tab.favicon })
      pushState()
      return
    }
    if (ctrl && input.key === 'h') { event.preventDefault(); win.webContents.send('open-history'); return }
    if (ctrl && input.key === 'j') { event.preventDefault(); win.webContents.send('open-downloads'); return }
  })
}

function setupContextMenu(view: WebContentsView, tab: Tab): void {
  view.webContents.on('context-menu', (_, p) => {
    const items: Electron.MenuItemConstructorOptions[] = []

    if (p.selectionText.trim()) {
      items.push({ label: 'Copier', role: 'copy' }, { type: 'separator' })
    }
    if (p.isEditable) {
      if (!p.selectionText.trim()) items.push({ label: 'Copier', role: 'copy' })
      items.push({ label: 'Couper', role: 'cut' }, { label: 'Coller', role: 'paste' }, { type: 'separator' })
    }
    if (p.linkURL) {
      items.push(
        { label: 'Ouvrir dans un nouvel onglet', click: () => createTab(p.linkURL) },
        { label: 'Copier l\'adresse du lien', click: () => clipboard.writeText(p.linkURL) },
        { type: 'separator' }
      )
    }
    items.push(
      { label: 'Précédent', enabled: view.webContents.navigationHistory.canGoBack(), click: () => view.webContents.navigationHistory.goBack() },
      { label: 'Suivant', enabled: view.webContents.navigationHistory.canGoForward(), click: () => view.webContents.navigationHistory.goForward() },
      { label: 'Actualiser', click: () => view.webContents.reload() },
      { type: 'separator' },
      {
        label: isBookmarked(tab.url) ? 'Retirer des favoris' : 'Ajouter aux favoris',
        click: () => { toggleBookmark({ url: tab.url, title: tab.title, favicon: tab.favicon }); pushState() }
      },
      { type: 'separator' },
      { label: 'Inspecter l\'élément', click: () => view.webContents.inspectElement(p.x, p.y) }
    )
    Menu.buildFromTemplate(items).popup({ window: win })
  })
}

function createTab(url = 'https://google.com'): Tab {
  const view = new WebContentsView({
    webPreferences: {
      contextIsolation: false,
      nodeIntegration: false,
      partition: BROWSER_PARTITION,
      preload: join(__dirname, '../preload/webview.js'),
    },
  })
  view.webContents.setUserAgent(CHROME_UA)

  const tab: Tab = { id: nextId++, view, title: 'New Tab', url, loading: true, canGoBack: false, canGoForward: false, zoom: 1.0 }
  tabs.push(tab)
  win.contentView.addChildView(view)

  view.webContents.setWindowOpenHandler(({ url: u, disposition }) => {
    if (!u || u === 'about:blank') return { action: 'deny' }
    if (sameDomain(u, tab.url)) {
      const t = createTab(u)
      if (disposition !== 'background-tab') switchTab(t.id)
      return { action: 'deny' }
    }
    const hostname = getHostname(u)
    if (settings.blockedDomains.includes(hostname) || settings.blockingMode === 'strict') return { action: 'deny' }
    hideActiveView()
    win.webContents.send('popup-confirm', { url: u, from: tab.url })
    return { action: 'deny' }
  })

  view.webContents.on('page-title-updated', (_, title) => { tab.title = title; pushState() })
  view.webContents.on('did-navigate', (_, u) => {
    tab.url = u
    tab.canGoBack = view.webContents.navigationHistory.canGoBack()
    tab.canGoForward = view.webContents.navigationHistory.canGoForward()
    addHistory({ url: u, title: tab.title, favicon: tab.favicon })
    pushState()
  })
  view.webContents.on('did-navigate-in-page', (_, u) => {
    tab.url = u
    tab.canGoBack = view.webContents.navigationHistory.canGoBack()
    tab.canGoForward = view.webContents.navigationHistory.canGoForward()
    pushState()
  })
  view.webContents.on('page-favicon-updated', (_, favs) => { tab.favicon = favs[0]; pushState() })
  view.webContents.on('did-start-loading', () => { tab.loading = true; pushState() })
  view.webContents.on('did-stop-loading', () => {
    tab.loading = false
    if (tab.url && !tab.url.startsWith('about:') && !tab.url.startsWith('devtools:')) {
      addHistory({ url: tab.url, title: tab.title, favicon: tab.favicon })
    }
    pushState()
  })
  view.webContents.on('enter-html-full-screen', () => {
    contentFullscreen = true; updateBounds(); win.webContents.send('content-fullscreen', true)
  })
  view.webContents.on('leave-html-full-screen', () => {
    contentFullscreen = false; updateBounds(); win.webContents.send('content-fullscreen', false)
  })
  view.webContents.on('found-in-page', (_, result) => {
    win.webContents.send('find-result', { activeMatch: result.activeMatchOrdinal, total: result.matches })
  })

  setupTabShortcuts(view, tab)
  setupContextMenu(view, tab)
  view.webContents.loadURL(url)
  return tab
}

function switchTab(id: number): void {
  if (contentFullscreen) { contentFullscreen = false; win.webContents.send('content-fullscreen', false) }
  if (findBarOpen) {
    getActive()?.view.webContents.stopFindInPage('clearSelection')
    findBarOpen = false
    win.webContents.send('find-force-close')
  }
  activeTabId = id
  updateBounds()
  pushState()
}

function closeTab(id: number): void {
  const idx = tabs.findIndex(t => t.id === id)
  if (idx === -1) return
  const [tab] = tabs.splice(idx, 1)
  win.contentView.removeChildView(tab.view)
  tab.view.webContents.close()
  if (tabs.length === 0) { const t = createTab(); switchTab(t.id) }
  else if (activeTabId === id) switchTab(tabs[Math.max(0, idx - 1)].id)
  pushState()
}

function setupWinShortcuts(): void {
  win.webContents.on('before-input-event', (event, input) => {
    if (input.type !== 'keyDown') return
    const ctrl = input.control || input.meta
    const shift = input.shift
    const tab = getActive()
    if (!tab) return

    if (ctrl && input.key === 't') { event.preventDefault(); const t = createTab(); switchTab(t.id) }
    else if (ctrl && input.key === 'w') { event.preventDefault(); closeTab(tab.id) }
    else if (ctrl && input.key === 'r') { event.preventDefault(); tab.view.webContents.reload() }
    else if (ctrl && (input.key === '+' || input.key === '=')) { event.preventDefault(); setZoom(tab, tab.zoom + 0.1) }
    else if (ctrl && input.key === '-') { event.preventDefault(); setZoom(tab, tab.zoom - 0.1) }
    else if (ctrl && input.key === '0') { event.preventDefault(); setZoom(tab, 1.0) }
    else if (ctrl && input.key === 'f') { event.preventDefault(); findBarOpen = true; updateBounds(); win.webContents.send('find-open') }
    else if (ctrl && !shift && input.key === 'Tab') {
      event.preventDefault()
      const idx = tabs.findIndex(t => t.id === activeTabId)
      switchTab(tabs[(idx + 1) % tabs.length].id)
    }
    else if (ctrl && shift && input.key === 'Tab') {
      event.preventDefault()
      const idx = tabs.findIndex(t => t.id === activeTabId)
      switchTab(tabs[(idx - 1 + tabs.length) % tabs.length].id)
    }
    else if (ctrl && input.key === 'd') {
      event.preventDefault()
      toggleBookmark({ url: tab.url, title: tab.title, favicon: tab.favicon })
      pushState()
    }
    else if (ctrl && input.key === 'h') { event.preventDefault(); win.webContents.send('open-history') }
    else if (ctrl && input.key === 'j') { event.preventDefault(); win.webContents.send('open-downloads') }
  })
}

function setupIPC(): void {
  ipcMain.handle('get-state', () => {
    const active = getActive()
    return {
      tabs: tabs.map(({ id, title, url, favicon, loading, canGoBack, canGoForward, zoom }) =>
        ({ id, title, url, favicon, loading, canGoBack, canGoForward, zoom })),
      activeTabId, adBlockerEnabled,
      isBookmarked: active ? isBookmarked(active.url) : false,
    }
  })
  ipcMain.handle('get-settings', () => settings)
  ipcMain.on('save-settings', (_, s: Settings) => { settings = s; saveSettings(s); pushSettings() })
  ipcMain.on('renderer-resize', (_, w: number, h: number) => updateBounds(w, h))
  ipcMain.on('navigate', (_, url: string) => getActive()?.view.webContents.loadURL(url))
  ipcMain.on('go-back', () => getActive()?.view.webContents.navigationHistory.goBack())
  ipcMain.on('go-forward', () => getActive()?.view.webContents.navigationHistory.goForward())
  ipcMain.on('reload', () => getActive()?.view.webContents.reload())
  ipcMain.on('new-tab', (_, url?: string) => { const t = createTab(url); switchTab(t.id) })
  ipcMain.on('close-tab', (_, id: number) => closeTab(id))
  ipcMain.on('switch-tab', (_, id: number) => switchTab(id))
  ipcMain.on('view-hide', () => hideActiveView())
  ipcMain.on('view-show', () => updateBounds())
  ipcMain.on('popup-open', (_, url: string) => { updateBounds(); const t = createTab(url); switchTab(t.id) })
  ipcMain.on('popup-block', (_, url: string) => {
    updateBounds()
    if (settings.autoBlockBlocked) {
      const h = getHostname(url)
      if (!settings.blockedDomains.includes(h)) {
        settings.blockedDomains = [...settings.blockedDomains, h]; saveSettings(settings); pushSettings()
      }
    }
  })
  ipcMain.on('popup-dismiss', () => updateBounds())
  ipcMain.on('toggle-adblock', () => {
    adBlockerEnabled = !adBlockerEnabled
    adBlockerEnabled
      ? blocker.enableBlockingInSession(session.defaultSession)
      : blocker.disableBlockingInSession(session.defaultSession)
    pushState()
  })
  ipcMain.on('zoom-in', () => { const t = getActive(); if (t) setZoom(t, t.zoom + 0.1) })
  ipcMain.on('zoom-out', () => { const t = getActive(); if (t) setZoom(t, t.zoom - 0.1) })
  ipcMain.on('zoom-reset', () => { const t = getActive(); if (t) setZoom(t, 1.0) })
  ipcMain.on('find-close', () => closeFindBar())
  ipcMain.on('find-search', (_, d: { text: string; forward: boolean }) => {
    if (!d.text) return
    getActive()?.view.webContents.findInPage(d.text, { forward: d.forward, findNext: false })
  })
  ipcMain.on('find-next', (_, d: { text: string; forward: boolean }) => {
    if (!d.text) return
    getActive()?.view.webContents.findInPage(d.text, { forward: d.forward, findNext: true })
  })

  // Bookmarks
  ipcMain.on('bookmark-toggle', () => {
    const t = getActive(); if (!t) return
    toggleBookmark({ url: t.url, title: t.title, favicon: t.favicon }); pushState()
  })
  ipcMain.handle('bookmark-list', () => getBookmarks())
  ipcMain.on('bookmark-delete', (_, url: string) => { deleteBookmark(url); pushState() })

  // History
  ipcMain.handle('history-list', (_, limit?: number, search?: string) => getHistory(limit, search))
  ipcMain.on('history-delete', (_, id: number) => deleteHistory(id))
  ipcMain.on('history-clear', () => clearHistory())

  // Downloads
  ipcMain.on('download-cancel', (_, id: number) => cancelDownload(id))
  ipcMain.on('download-open', (_, path: string) => openDownload(path))
  ipcMain.on('download-show-folder', (_, path: string) => showDownloadInFolder(path))
  ipcMain.handle('download-list-stored', () => getStoredDownloads())
  ipcMain.handle('downloads-get-active', () => getActiveDownloads())
}

async function createWindow(): Promise<void> {
  settings = loadSettings()

  win = new BrowserWindow({
    width: 1280, height: 800, minWidth: 800, minHeight: 600,
    frame: false, backgroundColor: '#0a0a0a',
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      contextIsolation: true, nodeIntegration: false,
    },
  })

  browserSession = session.fromPartition(BROWSER_PARTITION)
  browserSession.setUserAgent(CHROME_UA)
  blocker = await createAdBlocker(browserSession)

  browserSession.webRequest.onBeforeSendHeaders((details, callback) => {
    const h = { ...details.requestHeaders }
    h['User-Agent'] = CHROME_UA
    h['sec-ch-ua'] = '"Not(A:Brand";v="99", "Google Chrome";v="133", "Chromium";v="133"'
    h['sec-ch-ua-mobile'] = '?0'
    h['sec-ch-ua-platform'] = '"Linux"'
    h['sec-ch-ua-arch'] = '"x86"'
    h['sec-ch-ua-bitness'] = '"64"'
    h['sec-ch-ua-full-version-list'] = '"Not(A:Brand";v="99.0.0.0", "Google Chrome";v="133.0.6943.53", "Chromium";v="133.0.6943.53"'
    h['sec-ch-ua-model'] = '""'
    h['sec-ch-ua-platform-version'] = '"6.1.0"'
    h['sec-ch-ua-wow64'] = '?0'
    delete h['X-Requested-With']
    callback({ requestHeaders: h })
  })

  setupDownloads(browserSession, win)
  setupIPC()

  if (process.env.NODE_ENV === 'development') {
    win.loadURL('http://localhost:5173')
    win.webContents.openDevTools({ mode: 'detach' })
  } else {
    win.loadFile(join(__dirname, '../renderer/index.html'))
  }

  win.webContents.on('did-finish-load', () => {
    setupWinShortcuts()
    const t = createTab()
    switchTab(t.id)
  })
}

// Disable automation fingerprinting — prevents Google/Gmail "not secure" detection
app.commandLine.appendSwitch('disable-blink-features', 'AutomationControlled')
app.commandLine.appendSwitch('disable-features', 'AutofillEnableAccountStorage')
app.userAgentFallback = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36'

app.whenReady().then(createWindow)
app.on('window-all-closed', () => { if (process.platform !== 'darwin') app.quit() })
