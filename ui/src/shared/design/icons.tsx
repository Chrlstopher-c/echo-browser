// Responsabilite : jeu d'icones du chrome. Traits de 1.5px, grille 16, aucun remplissage.

import type { PropsWithChildren, ReactElement } from 'react'

export interface IconProps {
  size?: number
  className?: string
}

function Glyph({ size = 16, className, children }: PropsWithChildren<IconProps>): ReactElement {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.5}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      className={className}
    >
      {children}
    </svg>
  )
}

export function IconBack(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M10 3 5 8l5 5" />
    </Glyph>
  )
}

export function IconForward(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="m6 3 5 5-5 5" />
    </Glyph>
  )
}

export function IconReload(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M13.3 6.7A5.5 5.5 0 1 0 13.5 10" />
      <path d="M13.5 3v3.7h-3.7" />
    </Glyph>
  )
}

export function IconClose(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="m4.5 4.5 7 7M11.5 4.5l-7 7" />
    </Glyph>
  )
}

export function IconPlus(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M8 3.5v9M3.5 8h9" />
    </Glyph>
  )
}

export function IconShield(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M8 1.9 3.2 3.7v4.1c0 3 2 5.1 4.8 6.3 2.8-1.2 4.8-3.3 4.8-6.3V3.7z" />
    </Glyph>
  )
}

export function IconShieldCheck(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M8 1.9 3.2 3.7v4.1c0 3 2 5.1 4.8 6.3 2.8-1.2 4.8-3.3 4.8-6.3V3.7z" />
      <path d="m5.9 7.9 1.5 1.5 2.8-2.9" />
    </Glyph>
  )
}

export function IconMenu(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M3 4.5h10M3 8h10M3 11.5h10" />
    </Glyph>
  )
}

export function IconLock(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <rect x="3.5" y="7" width="9" height="6.2" rx="1.4" />
      <path d="M5.7 7V5.4a2.3 2.3 0 0 1 4.6 0V7" />
    </Glyph>
  )
}

export function IconWarning(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M8 2.4 1.9 13h12.2z" />
      <path d="M8 6.4v3.1M8 11.4h.01" />
    </Glyph>
  )
}

export function IconGlobe(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <circle cx="8" cy="8" r="5.6" />
      <path d="M2.4 8h11.2M8 2.4c1.5 1.6 2.3 3.5 2.3 5.6S9.5 12 8 13.6C6.5 12 5.7 10.1 5.7 8s.8-4 2.3-5.6" />
    </Glyph>
  )
}

export function IconStar(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="m8 2.3 1.75 3.6 3.95.57-2.85 2.8.67 3.95L8 11.35 4.48 13.2l.67-3.94L2.3 6.47l3.95-.57z" />
    </Glyph>
  )
}

export function IconClock(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <circle cx="8" cy="8" r="5.6" />
      <path d="M8 4.7V8l2.2 1.5" />
    </Glyph>
  )
}

export function IconDownload(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M8 2.6v7M5.2 7l2.8 2.7L10.8 7" />
      <path d="M2.9 11.7v1.1h10.2v-1.1" />
    </Glyph>
  )
}

export function IconPuzzle(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path
        d="M6.4 2.5h3.2v1.6a1.3 1.3 0 1 0 2.6 0h1.3v3.2h-1.6a1.3 1.3 0 1 0 0 2.6h1.6v3.6H9.9v-1.6
          a1.3 1.3 0 1 0-2.6 0v1.6H2.5V9.9h1.6a1.3 1.3 0 1 0 0-2.6H2.5V4.1h3.9z"
      />
    </Glyph>
  )
}

export function IconChevron(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="m6.5 4 4 4-4 4" />
    </Glyph>
  )
}
