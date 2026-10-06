// Responsabilite : bulles d'aide maison. Une seule couche ecoute le survol de toute la fenetre et
// remplace l'infobulle du systeme (attribut `title`) par une bulle de la matiere de la barre.
// Le raccourci en fin de libelle, « (Ctrl+R) », est rendu comme une touche.

import { AnimatePresence, motion } from 'framer-motion'
import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import type { ReactElement } from 'react'
import { QUICK } from './motion'

/** Delai avant apparition, et largeur sous laquelle la barre est trop etroite pour une bulle. */
const SHOW_DELAY_MS = 380
const MIN_VIEWPORT = 160
const EDGE = 8
const GAP = 8

interface Tip {
  text: string
  shortcut: string | null
  left: number
  top: number
  above: boolean
}

function split(label: string): { text: string; shortcut: string | null } {
  const match = /^(.*?)\s*\(([^()]+)\)$/.exec(label)
  return match === null ? { text: label, shortcut: null } : { text: match[1] ?? label, shortcut: match[2] ?? null }
}

function locate(target: Element, label: string): Tip {
  const box = target.getBoundingClientRect()
  const { text, shortcut } = split(label)
  const above = box.bottom + 40 > window.innerHeight
  return { text, shortcut, left: box.left + box.width / 2, top: above ? box.top - GAP : box.bottom + GAP, above }
}

function useTip(): Tip | null {
  const [tip, setTip] = useState<Tip | null>(null)
  useEffect(() => {
    let timer = 0
    const hide = (): void => {
      window.clearTimeout(timer)
      setTip(null)
    }
    const onOver = (event: PointerEvent): void => {
      const holder = (event.target as Element | null)?.closest('[title], [data-tip]')
      if (holder === null || holder === undefined || window.innerWidth < MIN_VIEWPORT) return hide()
      const label = holder.getAttribute('title') ?? holder.getAttribute('data-tip') ?? ''
      if (label === '') return hide()
      holder.setAttribute('data-tip', label)
      holder.removeAttribute('title')
      window.clearTimeout(timer)
      timer = window.setTimeout(() => setTip(locate(holder, label)), SHOW_DELAY_MS)
    }
    document.addEventListener('pointerover', onOver)
    document.addEventListener('pointerdown', hide)
    document.addEventListener('pointerleave', hide)
    window.addEventListener('blur', hide)
    return () => {
      hide()
      document.removeEventListener('pointerover', onOver)
      document.removeEventListener('pointerdown', hide)
      document.removeEventListener('pointerleave', hide)
      window.removeEventListener('blur', hide)
    }
  }, [])
  return tip
}

/** Recale la bulle dans la fenetre une fois sa largeur connue. */
function useKeepInside(tip: Tip | null) {
  const ref = useRef<HTMLDivElement>(null)
  useLayoutEffect(() => {
    const node = ref.current
    if (node === null) return
    const { left, right } = node.getBoundingClientRect()
    const shift = left < EDGE ? EDGE - left : right > window.innerWidth - EDGE ? window.innerWidth - EDGE - right : 0
    if (shift !== 0) node.style.left = `${parseFloat(node.style.left) + shift}px`
  }, [tip])
  return ref
}

export function TooltipLayer(): ReactElement {
  const tip = useTip()
  const ref = useKeepInside(tip)
  return (
    <AnimatePresence>
      {tip !== null && (
        <div
          ref={ref}
          key={`${tip.left}-${tip.top}`}
          className="pointer-events-none fixed z-50"
          style={{
            left: tip.left,
            top: tip.top,
            transform: `translate(-50%, ${tip.above ? '-100%' : '0'})`,
            maxWidth: window.innerWidth - EDGE * 2,
          }}
        >
          <motion.div
            initial={{ opacity: 0, y: tip.above ? 3 : -3 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            transition={QUICK}
            className="flex items-center gap-2 rounded-row bg-card px-2.5 py-1.5 text-[12px] text-ink shadow-lift"
          >
            <span>{tip.text}</span>
            {tip.shortcut !== null && (
              <kbd className="numerique rounded-[5px] bg-field px-1.5 py-px text-[10.5px] text-ink-muted">
                {tip.shortcut}
              </kbd>
            )}
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  )
}
