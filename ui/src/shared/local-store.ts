// Responsabilite : lecture et ecriture d'une preference locale. Ne jette jamais : sans stockage
// (page interne, stockage bloque), l'interface repart de ses valeurs par defaut.

export function readLocal<T>(key: string, parse: (raw: unknown) => T | null): T | null {
  try {
    const raw = window.localStorage.getItem(key)
    if (raw === null) return null
    return parse(JSON.parse(raw))
  } catch (error) {
    console.warn(`[echo] lecture locale impossible pour ${key}`, error)
    return null
  }
}

export function writeLocal(key: string, value: unknown): void {
  try {
    window.localStorage.setItem(key, JSON.stringify(value))
  } catch (error) {
    console.warn(`[echo] ecriture locale impossible pour ${key}`, error)
  }
}
