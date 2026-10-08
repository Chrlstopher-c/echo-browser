// Responsabilite : jeu d'icones du chrome. Traits de 1.5px, grille 16, aucun remplissage sauf mention.

import type { PropsWithChildren, ReactElement } from 'react'

export interface IconProps {
  size?: number
  className?: string
}

export type IconComponent = (props: IconProps) => ReactElement

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

export function IconStop(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <rect x="4.5" y="4.5" width="7" height="7" rx="1.2" fill="currentColor" stroke="none" />
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

export function IconMinus(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M3.5 8h9" />
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

export function IconLock(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <rect x="3.5" y="7" width="9" height="6.2" rx="1.4" />
      <path d="M5.7 7V5.4a2.3 2.3 0 0 1 4.6 0V7" />
    </Glyph>
  )
}

export function IconLockOpen(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <rect x="3.5" y="7" width="9" height="6.2" rx="1.4" />
      <path d="M5.7 7V5.4a2.3 2.3 0 0 1 4.4-.9" />
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

export function IconStarFilled(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path
        d="m8 2.3 1.75 3.6 3.95.57-2.85 2.8.67 3.95L8 11.35 4.48 13.2l.67-3.94L2.3 6.47l3.95-.57z"
        fill="currentColor"
      />
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

export function IconLibrary(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M3 3.2v9.6M6.2 3.2v9.6" />
      <path d="m9 3.9 3.6-.9 2 8.9-3.6.9z" />
    </Glyph>
  )
}

export function IconTerminal(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="m3.5 4.8 3.2 3.2-3.2 3.2" />
      <path d="M8.6 11.6h4" />
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

export function IconChevronLeft(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="m9.5 4-4 4 4 4" />
    </Glyph>
  )
}

export function IconChevronDown(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="m4 6.5 4 4 4-4" />
    </Glyph>
  )
}

export function IconSidebar(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <rect x="2.5" y="3" width="11" height="10" rx="2" />
      <path d="M6.5 3v10" />
    </Glyph>
  )
}

export function IconSettings(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M3 5h10M3 11h10" />
      <circle cx="6" cy="5" r="1.6" fill="var(--color-shell)" />
      <circle cx="10" cy="11" r="1.6" fill="var(--color-shell)" />
    </Glyph>
  )
}

export function IconPin(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M9.6 2.6 13.4 6.4 11.2 7l-2 2-.4 3.4-4.2-4.2L8 7.8l-.4-2.6z" />
      <path d="M6.4 9.6 3 13" />
    </Glyph>
  )
}

export function IconUnpin(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M9.6 2.6 13.4 6.4 11.2 7l-2 2-.4 3.4-4.2-4.2L8 7.8l-.4-2.6z" />
      <path d="M6.4 9.6 3 13M2.5 2.5l11 11" />
    </Glyph>
  )
}

export function IconCheck(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="m3.5 8.3 2.8 2.8 6.2-6.4" />
    </Glyph>
  )
}

export function IconTrash(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M3 4.3h10M6.4 4.3V2.9h3.2v1.4" />
      <path d="M4.4 4.3l.6 8.2c0 .4.4.6.8.6h4.4c.4 0 .8-.2.8-.6l.6-8.2" />
    </Glyph>
  )
}

export function IconSearch(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <circle cx="7" cy="7" r="4.2" />
      <path d="m10.2 10.2 3.3 3.3" />
    </Glyph>
  )
}

export function IconFolder(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M2.5 4.5a1 1 0 0 1 1-1h3l1.5 1.5h4.5a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1h-9a1 1 0 0 1-1-1z" />
    </Glyph>
  )
}

export function IconUser(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <circle cx="8" cy="5.5" r="2.5" />
      <path d="M3 13.5c.4-2.6 2.4-4 5-4s4.6 1.4 5 4" />
    </Glyph>
  )
}

export function IconOpen(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M7 3.5H4.2a1 1 0 0 0-1 1v7.3a1 1 0 0 0 1 1h7.3a1 1 0 0 0 1-1V9" />
      <path d="M9.3 2.8h4v4M13.2 2.9 7.6 8.5" />
    </Glyph>
  )
}

export function IconMoon(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M12.8 9.6A5.2 5.2 0 0 1 6.4 3.2a5.2 5.2 0 1 0 6.4 6.4z" />
    </Glyph>
  )
}

export function IconVolume(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M3 6.2h2.3L8.5 3.6v8.8L5.3 9.8H3z" />
      <path d="M10.6 5.8a3 3 0 0 1 0 4.4M12.4 4a5.6 5.6 0 0 1 0 8" />
    </Glyph>
  )
}

export function IconVolumeOff(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M3 6.2h2.3L8.5 3.6v8.8L5.3 9.8H3z" />
      <path d="m10.6 6.4 3 3.2M13.6 6.4l-3 3.2" />
    </Glyph>
  )
}

export function IconMore(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <circle cx="4" cy="8" r="0.9" fill="currentColor" />
      <circle cx="8" cy="8" r="0.9" fill="currentColor" />
      <circle cx="12" cy="8" r="0.9" fill="currentColor" />
    </Glyph>
  )
}

export function IconZoom(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <circle cx="7" cy="7" r="4.2" />
      <path d="m10.2 10.2 3.3 3.3M5 7h4M7 5v4" />
    </Glyph>
  )
}

export function IconExitFullscreen(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M6.5 2.5v4h-4M9.5 2.5v4h4M6.5 13.5v-4h-4M9.5 13.5v-4h4" />
    </Glyph>
  )
}

export function IconRefresh(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M2.8 8a5.2 5.2 0 0 1 9.1-3.4M13.2 8a5.2 5.2 0 0 1-9.1 3.4" />
      <path d="M12 2v2.8H9.2M4 14v-2.8h2.8" />
    </Glyph>
  )
}

export function IconFile(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M4 2.5h5l3.5 3.5v7a1 1 0 0 1-1 1h-7.5a1 1 0 0 1-1-1v-9.5a1 1 0 0 1 1-1z" />
      <path d="M9 2.5V6h3.5" />
    </Glyph>
  )
}

export function IconSparkle(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M8 2.5c.4 2.9 2.2 4.7 5.5 5.5-3.3.8-5.1 2.6-5.5 5.5-.4-2.9-2.2-4.7-5.5-5.5 3.3-.8 5.1-2.6 5.5-5.5z" />
    </Glyph>
  )
}

export function IconWrench(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path
        d="M13.2 4.6a3.3 3.3 0 0 1-4.4 3.9L4 13.3a1.4 1.4 0 0 1-2-2l4.8-4.8a3.3 3.3 0 0 1 3.9-4.4L8.9 3.9l1.2 2 2-.2z"
      />
    </Glyph>
  )
}

export function IconSun(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <circle cx="8" cy="8" r="2.6" />
      <path d="M8 2v1.4M8 12.6V14M2 8h1.4M12.6 8H14M3.8 3.8l1 1M11.2 11.2l1 1M12.2 3.8l-1 1M4.8 11.2l-1 1" />
    </Glyph>
  )
}

export function IconCopy(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <rect x="5.5" y="5.5" width="8" height="8" rx="1.6" />
      <path d="M10.5 5.5V3.9c0-.8-.6-1.4-1.4-1.4H3.9c-.8 0-1.4.6-1.4 1.4v5.2c0 .8.6 1.4 1.4 1.4h1.6" />
    </Glyph>
  )
}

export function IconClipboard(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <rect x="3.2" y="3.2" width="9.6" height="10.6" rx="1.6" />
      <path d="M6 2.2h4v2H6z" />
    </Glyph>
  )
}

export function IconScissors(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <circle cx="4.3" cy="11.6" r="1.8" />
      <circle cx="11.7" cy="11.6" r="1.8" />
      <path d="M5.6 10.3 12 2.8M10.4 10.3 4 2.8" />
    </Glyph>
  )
}

export function IconPrinter(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M4.5 5.8V2.6h7v3.2M4.5 11.2H3a1 1 0 0 1-1-1V6.8a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v3.4a1 1 0 0 1-1 1h-1.5" />
      <path d="M4.5 9.2h7v4.2h-7z" />
    </Glyph>
  )
}

export function IconCode(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M5.5 4.5 2 8l3.5 3.5M10.5 4.5 14 8l-3.5 3.5" />
    </Glyph>
  )
}

export function IconImage(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <rect x="2.2" y="2.8" width="11.6" height="10.4" rx="1.8" />
      <circle cx="5.8" cy="6.2" r="1.1" />
      <path d="m13.8 10.4-3.3-3.2-6.8 6" />
    </Glyph>
  )
}

/** Activite reseau : une trace de pouls. */
export function IconActivity(props: IconProps): ReactElement {
  return (
    <Glyph {...props}>
      <path d="M1.5 8h2.7l1.8-4.6 3.3 9.2 1.8-4.6h3.4" />
    </Glyph>
  )
}
