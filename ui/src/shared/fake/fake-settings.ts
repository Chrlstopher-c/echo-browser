// Responsabilite : volet reglages du faux coeur — un jeu de cles typees, modifiables, rien de persiste.
// Les cles reprennent celles que le vrai coeur livre ; l'interface les habille (settings/setting-catalogue).

import type { CoreEvent, SettingView, UiRequest } from '../contract'

const SEED: SettingView[] = [
  { key: 'search.engine', value: { type: 'text', value: 'https://www.google.com/search?q=%s' } },
  { key: 'home.url', value: { type: 'text', value: 'https://www.google.com/' } },
  { key: 'session.restore', value: { type: 'flag', value: true } },
  { key: 'newtab.focusAddress', value: { type: 'flag', value: true } },
  { key: 'privacy.doNotTrack', value: { type: 'flag', value: true } },
  { key: 'privacy.thirdPartyCookies', value: { type: 'flag', value: false } },
  { key: 'privacy.clearOnExit', value: { type: 'flag', value: false } },
  { key: 'privacy.httpsOnly', value: { type: 'flag', value: true } },
  { key: 'page.defaultZoom', value: { type: 'number', value: 100 } },
  { key: 'page.minimumFontSize', value: { type: 'number', value: 0 } },
  { key: 'page.smoothScrolling', value: { type: 'flag', value: true } },
  { key: 'tabs.sleepEnabled', value: { type: 'flag', value: true } },
  { key: 'tabs.sleepAfterMinutes', value: { type: 'number', value: 30 } },
  { key: 'tabs.confirmCloseMany', value: { type: 'flag', value: true } },
  { key: 'downloads.directory', value: { type: 'text', value: '/home/chris/Téléchargements' } },
  { key: 'downloads.askWhere', value: { type: 'flag', value: false } },
  { key: 'system.hardwareAcceleration', value: { type: 'flag', value: true } },
  { key: 'system.devTools', value: { type: 'flag', value: true } },
  { key: 'experimental.webgpu', value: { type: 'flag', value: false } },
]

type Emit = (event: CoreEvent) => void

export class FakeSettings {
  private items: SettingView[] = [...SEED]

  constructor(private readonly emit: Emit) {}

  public snapshot(): CoreEvent {
    return { kind: 'settingsChanged', settings: this.items }
  }

  public handle(request: UiRequest): boolean {
    if (request.kind !== 'updateSetting') return false
    const known = this.items.some((item) => item.key === request.key)
    this.items = known
      ? this.items.map((item) => (item.key === request.key ? { key: item.key, value: request.value } : item))
      : [...this.items, { key: request.key, value: request.value }]
    this.emit(this.snapshot())
    return true
  }
}
