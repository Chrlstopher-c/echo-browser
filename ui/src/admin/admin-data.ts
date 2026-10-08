// Responsabilite : lire le tableau de bord rendu par le service (forme libre cote contrat) en types surs, sans
// conversion forcee : une valeur absente ou d'un autre type devient 0, '' ou null.

export interface DayCount {
  day: string
  count: number
  errors: number
}

export interface NamedCount {
  name: string
  count: number
}

export interface AdminSummary {
  accounts: number
  active: { day: number; week: number; month: number }
  sessions: number
  connected: number
  loginFailures: number
  signups: DayCount[]
  requests: DayCount[]
  versions: NamedCount[]
  storage: Array<NamedCount & { bytes: number }>
  routes: NamedCount[]
  activeByDay: DayCount[]
  syncsByDay: DayCount[]
  writesByType: NamedCount[]
  topUsers: Array<NamedCount & { id: string }>
  machines: { total: number; active: number }
}

export interface AdminMachine {
  createdAt: number
  seenAt: number
  version: string | null
  /** Fin de la session, null si elle est fermee. */
  expiresAt: number | null
}

export interface AdminDetail {
  account: AdminAccount
  byDay: DayCount[]
  byAction: NamedCount[]
  machines: AdminMachine[]
  vault: Array<{ type: string; version: number; bytes: number; updatedAt: number }>
}

export interface AdminAccount {
  id: string
  email: string
  createdAt: number
  seenAt: number | null
  version: string | null
  bytes: number
  sessions: number
  admin: boolean
}

type Fields = Record<string, unknown>

function fields(value: unknown): Fields {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) return {}
  return Object.fromEntries(Object.entries(value))
}

function list(value: unknown): Fields[] {
  return Array.isArray(value) ? value.map(fields) : []
}

const num = (value: unknown): number => (typeof value === 'number' && Number.isFinite(value) ? value : 0)
const str = (value: unknown): string => (typeof value === 'string' ? value : '')
const maybeNum = (value: unknown): number | null => (typeof value === 'number' ? value : null)
const maybeStr = (value: unknown): string | null => (typeof value === 'string' ? value : null)

export function readSummary(raw: unknown): AdminSummary | null {
  const r = fields(raw)
  if (Object.keys(r).length === 0) return null
  const active = fields(r['actifs'])
  const sessions = fields(r['sessions'])
  return {
    accounts: num(fields(r['comptes'])['n']),
    active: { day: num(active['j1']), week: num(active['j7']), month: num(active['j30']) },
    sessions: num(sessions['n']),
    connected: num(sessions['comptes']),
    loginFailures: num(fields(r['echecsConnexion24h'])['n']),
    signups: list(r['inscriptions']).map((d) => ({ day: str(d['jour']), count: num(d['n']), errors: 0 })),
    requests: list(r['requetes']).map((d) => ({ day: str(d['jour']), count: num(d['n']), errors: num(d['erreurs']) })),
    versions: list(r['versions']).map((v) => ({ name: str(v['version']), count: num(v['n']) })),
    storage: list(r['coffre']).map((c) => ({ name: str(c['type']), count: num(c['n']), bytes: num(c['octets']) })),
    routes: list(r['routes']).map((x) => ({ name: str(x['route']), count: num(x['n']) })),
    activeByDay: days(r['actifsParJour']),
    syncsByDay: days(r['synchros']),
    writesByType: list(r['ecritures']).map((x) => ({ name: str(x['type']), count: num(x['n']) })),
    topUsers: list(r['plusActifs']).map((x) => ({ id: str(x['id']), name: str(x['email']), count: num(x['n']) })),
    machines: { total: num(fields(r['machines'])['n']), active: num(fields(r['machines'])['actives']) },
  }
}

function days(raw: unknown): DayCount[] {
  return list(raw).map((d) => ({ day: str(d['jour']), count: num(d['n']), errors: 0 }))
}

export function readDetail(raw: unknown): AdminDetail | null {
  const r = fields(raw)
  const account = readAccounts([r['compte']])[0]
  if (account === undefined || account.id === '') return null
  return {
    account,
    byDay: days(r['parJour']),
    byAction: list(r['parAction']).map((x) => ({ name: str(x['action']), count: num(x['n']) })),
    machines: list(r['machines']).map((m) => ({
      createdAt: num(m['creeLe']),
      seenAt: num(m['vuLe']),
      version: maybeStr(m['version']),
      expiresAt: maybeNum(m['expireLe']),
    })),
    vault: list(r['coffre']).map((c) => ({
      type: str(c['type']), version: num(c['version']), bytes: num(c['octets']), updatedAt: num(c['majLe']),
    })),
  }
}

export function readAccounts(raw: unknown): AdminAccount[] {
  return list(raw).map((c) => ({
    id: str(c['id']),
    email: str(c['email']),
    createdAt: num(c['creeLe']),
    seenAt: maybeNum(c['vuLe']),
    version: maybeStr(c['version']),
    bytes: num(c['octets']),
    sessions: num(c['sessions']),
    admin: c['admin'] === 1 || c['admin'] === true,
  }))
}

/** Signaux anonymes agreges : seules les cles vues par au moins `threshold` installations un meme jour. */
export interface AdminSignals {
  threshold: number
  batchesByDay: DayCount[]
  versions: NamedCount[]
  sites: SignalKey[]
  blocked: SignalKey[]
}

export interface SignalKey {
  key: string
  total: number
  /** Installations le jour ou elles etaient le plus nombreuses. */
  installs: number
  days: number
}

function keys(raw: unknown): SignalKey[] {
  return list(raw).map((k) => ({ key: str(k['cle']), total: num(k['total']), installs: num(k['installs']),
    days: num(k['jours']) }))
}

export function readSignals(raw: unknown): AdminSignals | null {
  const r = fields(raw)
  if (Object.keys(r).length === 0) return null
  return {
    threshold: num(r['seuil']),
    batchesByDay: days(r['lotsParJour']),
    versions: list(r['versions']).map((v) => ({ name: str(v['version']), count: num(v['n']) })),
    sites: keys(r['sites']),
    blocked: keys(r['bloques']),
  }
}
