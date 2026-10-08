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

const KIND_LABEL: Record<string, string> = {
  reglages: 'Réglages', favoris: 'Favoris', extensions: 'Extensions', onglets: 'Onglets ouverts',
  historique: 'Historique',
}

export function kindLabel(kind: string): string {
  return KIND_LABEL[kind] ?? kind
}

/** Une action du service en francais : `ecriture:favoris` → « Envoi : Favoris ». */
export function actionLabel(action: string): string {
  if (action.startsWith('ecriture:')) return `Envoi : ${kindLabel(action.slice(9))}`
  const known: Record<string, string> = {
    synchro: 'Synchronisation', connexion: 'Connexion', inscription: 'Inscription', moi: 'Vérification du compte',
    deconnexion: 'Déconnexion', administration: 'Administration', compte: 'Suppression du compte',
  }
  return known[action] ?? action
}
