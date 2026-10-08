// Responsabilite : contrat du compte Echo (miroir de core/contract/src/account.rs).

/** Le compte Echo de cette machine. */
export interface AccountView {
  /** Un service de compte est configure. */
  available: boolean
  email: string | null
  /** Connexion ou synchronisation en cours. */
  busy: boolean
  /** Derniere synchronisation reussie (secondes Unix). */
  lastSync: number | null
  error: string | null
  /** L'historique est synchronise (reglage `sync.history`). */
  history: boolean
  /** Le compte ouvre l'administration (drapeau pose sur le serveur). */
  admin: boolean
  /** Mode de synchronisation (reglage `sync.mode`). */
  mode: SyncMode
  /** Des modifications locales attendent d'etre envoyees. */
  pending: boolean
}

export type SyncMode = 'realtime' | 'auto' | 'manual'

/** Un type de donnees tel que le service le garde. */
export interface VaultKindView {
  /** `reglages`, `favoris`, `extensions`, `onglets`, `historique`. */
  kind: string
  version: number
  /** Taille chiffree sur le service, en octets. */
  bytes: number
  /** Derniere ecriture (millisecondes Unix). */
  updated: number
  count: number
  /** Les elements lisibles (les premiers seulement). */
  lines: Array<{ title: string; detail: string }>
}

export interface RemoteMachineView {
  name: string
  /** Derniere publication de ses onglets (secondes Unix). */
  updated: number
  tabs: Array<{ url: string; title: string }>
}
