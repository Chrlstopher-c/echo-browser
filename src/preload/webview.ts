// Runs in main world of every tab — BEFORE page JS
// contextIsolation: false required so this patches the actual window object

Object.defineProperty(navigator, 'webdriver', { get: () => undefined, configurable: true })

// Override userAgentData brands — Electron exposes "Electron" brand which Google detects
const UA_BRANDS = [
  { brand: 'Not(A:Brand', version: '99' },
  { brand: 'Google Chrome', version: '133' },
  { brand: 'Chromium', version: '133' },
]
const HIGH_ENTROPY: Record<string, unknown> = {
  architecture: 'x86',
  bitness: '64',
  brands: UA_BRANDS,
  fullVersionList: [
    { brand: 'Not(A:Brand', version: '99.0.0.0' },
    { brand: 'Google Chrome', version: '133.0.6943.53' },
    { brand: 'Chromium', version: '133.0.6943.53' },
  ],
  mobile: false,
  model: '',
  platform: 'Linux',
  platformVersion: '6.1.0',
  uaFullVersion: '133.0.6943.53',
}
const uaData = {
  brands: UA_BRANDS,
  mobile: false,
  platform: 'Linux',
  getHighEntropyValues: (hints: string[]) => {
    const result: Record<string, unknown> = {}
    for (const h of hints) if (h in HIGH_ENTROPY) result[h] = HIGH_ENTROPY[h]
    return Promise.resolve(result)
  },
  toJSON: () => ({ brands: UA_BRANDS, mobile: false, platform: 'Linux' }),
}
try {
  Object.defineProperty(Navigator.prototype, 'userAgentData', { get: () => uaData, configurable: true })
} catch {
  Object.defineProperty(navigator, 'userAgentData', { get: () => uaData, configurable: true })
}

if (!window.chrome) {
  Object.defineProperty(window, 'chrome', {
    configurable: true,
    writable: false,
    value: {
      app: {
        isInstalled: false,
        InstallState: { DISABLED: 'disabled', INSTALLED: 'installed', NOT_INSTALLED: 'not_installed' },
        RunningState: { CANNOT_RUN: 'cannot_run', READY_TO_RUN: 'ready_to_run', RUNNING: 'running' },
      },
      webstore: { onInstallStageChanged: {}, onDownloadProgress: {} },
      runtime: {
        PlatformOs: { MAC: 'mac', WIN: 'win', ANDROID: 'android', CROS: 'cros', LINUX: 'linux', OPENBSD: 'openbsd' },
        PlatformArch: { ARM: 'arm', X86_32: 'x86-32', X86_64: 'x86-64' },
        RequestUpdateCheckStatus: { THROTTLED: 'throttled', NO_UPDATE: 'no_update', UPDATE_AVAILABLE: 'update_available' },
        OnInstalledReason: { INSTALL: 'install', UPDATE: 'update', CHROME_UPDATE: 'chrome_update', SHARED_MODULE_UPDATE: 'shared_module_update' },
        OnRestartRequiredReason: { APP_UPDATE: 'app_update', OS_UPDATE: 'os_update', PERIODIC: 'periodic' },
        connect: () => {},
        sendMessage: () => {},
        onMessage: { addListener: () => {}, removeListener: () => {} },
        onConnect: { addListener: () => {}, removeListener: () => {} },
      },
      loadTimes: () => ({}),
      csi: () => ({}),
    },
  })
}
