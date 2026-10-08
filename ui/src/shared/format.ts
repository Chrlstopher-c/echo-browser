// Responsabilite : mise en forme lisible des nombres, tailles et dates de l'interface, en francais.

const DAY_MS = 86_400_000

/** Le coeur horodate en secondes Unix ; l'affichage raisonne en millisecondes. */
export function fromCoreTime(seconds: number): number {
  return seconds * 1000
}

/** Compteur lisible : espace fine insecable tous les trois chiffres. */
export function formatCount(value: number): string {
  return value.toLocaleString('fr-FR').replace(/ | /g, ' ')
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} o`
  const units = ['Ko', 'Mo', 'Go', 'To']
  let value = bytes / 1024
  let index = 0
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024
    index += 1
  }
  const digits = value >= 100 ? 0 : 1
  return `${value.toFixed(digits).replace('.', ',')} ${units[index]}`
}

export function formatPercent(ratio: number): string {
  return `${Math.round(Math.min(Math.max(ratio, 0), 1) * 100)} %`
}

export function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' })
}

/** Debut du jour local d'un horodatage, cle de regroupement. */
export function startOfDay(timestamp: number): number {
  const date = new Date(timestamp)
  date.setHours(0, 0, 0, 0)
  return date.getTime()
}

/** « Aujourd'hui », « Hier », sinon la date longue. */
export function formatDay(dayStart: number, now: number = Date.now()): string {
  const today = startOfDay(now)
  if (dayStart === today) return "Aujourd'hui"
  if (dayStart === today - DAY_MS) return 'Hier'
  const date = new Date(dayStart)
  const sameYear = date.getFullYear() === new Date(now).getFullYear()
  const label = date.toLocaleDateString('fr-FR', {
    weekday: 'long',
    day: 'numeric',
    month: 'long',
    ...(sameYear ? {} : { year: 'numeric' }),
  })
  return label.charAt(0).toUpperCase() + label.slice(1)
}

/** « à l'instant », « il y a 3 min », « il y a 2 h », « hier », sinon la date courte. */
export function formatRelative(timestamp: number, now: number = Date.now()): string {
  const elapsed = Math.max(0, now - timestamp)
  const minutes = Math.round(elapsed / 60_000)
  if (minutes < 1) return "à l'instant"
  if (minutes < 60) return `il y a ${minutes} min`
  const hours = Math.round(minutes / 60)
  if (hours < 24) return `il y a ${hours} h`
  const days = Math.round(hours / 24)
  if (days === 1) return 'hier'
  if (days < 7) return `il y a ${days} jours`
  return new Date(timestamp).toLocaleDateString('fr-FR', { day: 'numeric', month: 'short' })
}

export function formatZoom(factor: number): string {
  return `${Math.round(factor * 100)} %`
}
