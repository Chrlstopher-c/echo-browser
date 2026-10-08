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
  return typeof value === 'object' && value !== null && !Array.isArray(value) ? Object.fromEntries(Object.entries(value)) : {}
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
