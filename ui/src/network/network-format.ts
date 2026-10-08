// Responsabilite : mise en forme du panneau Reseau — tailles, durees, types et raisons de blocage en francais.

export function size(bytes: number): string {
  if (bytes < 1024) return `${bytes} o`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} Ko`
  return `${(bytes / 1024 / 1024).toFixed(1)} Mo`
}

const KIND: Record<string, string> = {
  main_frame: 'page', sub_frame: 'cadre', script: 'script', stylesheet: 'style', image: 'image', font: 'police',
  media: 'média', xmlhttprequest: 'requête', ping: 'mesure', websocket: 'socket', other: 'autre',
}

export function kindLabel(kind: string): string {
  return KIND[kind] ?? kind
}

const BLOCKED: Record<string, string> = {
  bouclier: 'bloquée par le bouclier', regle: 'bloquée par votre règle', isolement: 'bloquée par l’isolement',
}

export function blockedLabel(reason: string): string {
  return BLOCKED[reason] ?? 'bloquée'
}
