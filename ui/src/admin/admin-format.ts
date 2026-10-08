// Responsabilite : mise en forme des chiffres du tableau de bord (tailles, dates, nombres).

export function bytes(value: number): string {
  if (value < 1024) return `${value} o`
  if (value < 1024 * 1024) return `${Math.round(value / 1024)} Ko`
  return `${(value / 1024 / 1024).toFixed(1)} Mo`
}

export function count(value: number): string {
  return new Intl.NumberFormat('fr-FR').format(value)
}

export function when(ms: number | null): string {
  if (ms === null || ms === 0) return '—'
  return new Date(ms).toLocaleString('fr-FR', { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' })
}
