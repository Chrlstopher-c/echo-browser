// Responsabilite : modele des extensions. Le coeur ne les expose pas encore dans le contrat.

export interface Extension {
  id: string
  name: string
  version: string
  enabled: boolean
  /** Description courte, une ligne. */
  summary: string
}

export const NO_EXTENSIONS: Extension[] = []

/** Un changement d'extension ne prend effet qu'apres relance du navigateur. */
export const RESTART_NOTICE = "L'ajout ou le retrait d'une extension prend effet au prochain démarrage."
