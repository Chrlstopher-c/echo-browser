// Responsabilite : contrat du panneau Reseau et des routines (miroir de core/contract/src/network.rs).

/** Ce que charge l'onglet actif, resume par domaine, et les regles du site. */
export interface NetworkView {
  page: string
  totalBytes: number
  domains: NetDomainView[]
  /** Dernieres requetes (du domaine choisi, ou toutes), les plus recentes d'abord. */
  requests: NetRequestView[]
  focus: string | null
  /** Hotes bloques par l'utilisateur sur ce site. */
  blockedHosts: string[]
  /** Isolement strict du site : aucune requete tierce. */
  strict: boolean
  /** Journal d'acces du site, le plus recent d'abord. */
  journal: JournalEntryView[]
}

export interface JournalEntryView {
  /** Secondes Unix. */
  at: number
  /** `tiers`, `permission`, `telechargement`. */
  kind: string
  detail: string
}

export interface NetDomainView {
  host: string
  site: string
  requests: number
  bytes: number
  blocked: number
  thirdParty: boolean
  /** Requetes par type, les plus nombreuses d'abord : [type, nombre]. */
  kinds: Array<[string, number]>
}

export interface NetRequestView {
  url: string
  host: string
  kind: string
  method: string
  thirdParty: boolean
  /** `bouclier`, `regle` ou `isolement` quand elle a ete bloquee. */
  blocked: string | null
  status: number | null
  bytes: number
  durationMs: number | null
}

/** Une suite de sites ouverte souvent dans le meme ordre, proposee comme routine. */
export interface RoutineProposalView {
  fingerprint: string
  sites: string[]
  urls: string[]
}

export interface RoutineView {
  id: number
  name: string
  urls: string[]
}
