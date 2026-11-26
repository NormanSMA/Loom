import { useEffect, useState } from 'react'
import type { AgentActivityState } from '../types/api'

interface AgentGlyphProps {
  state: AgentActivityState
  size?: 'sm' | 'md'
}

const DOT_MATRIX = [0, 1, 2, 3, 4, 5, 6, 7, 8] as const

const DELAY_CLASSES = [
  'glyph-delay-0',
  'glyph-delay-1',
  'glyph-delay-2',
  'glyph-delay-3',
  'glyph-delay-4',
  'glyph-delay-5',
  'glyph-delay-6',
  'glyph-delay-7',
  'glyph-delay-8',
] as const

const READING_TRIANGLE = new Set([1, 6, 8])
const RING_INDICES = new Set([0, 1, 2, 3, 5, 6, 7, 8])
const DEBUG_DIAGONAL = new Set([0, 1, 3, 4, 5, 7, 8])
const CORNER_INDICES = new Set([0, 2, 6, 8])
const DIAGONAL_ORDER = [0, 1, 2, 1, 2, 3, 2, 3, 4] as const
const ORBIT_ORDER = [0, 1, 2, 7, 4, 3, 6, 5, 4] as const

function isCellActive(state: AgentActivityState, idx: number): boolean {
  switch (state) {
    case 'idle':
      return idx === 4
    case 'reading':
      return READING_TRIANGLE.has(idx)
    case 'analysing':
      return RING_INDICES.has(idx)
    case 'debugging':
      return DEBUG_DIAGONAL.has(idx)
    case 'thinking':
    case 'creating':
      return true
  }
}

function cellColorClass(state: AgentActivityState, idx: number): string {
  switch (state) {
    case 'thinking':
      return 'bg-[var(--accent-green)]'
    case 'reading':
      return 'bg-[var(--accent-red)]'
    case 'analysing':
    case 'debugging':
      return 'bg-[var(--accent-blue)]'
    case 'creating':
      return CORNER_INDICES.has(idx) || idx === 4
        ? 'bg-[var(--accent-green)]'
        : 'bg-[var(--accent-blue)]'
    case 'idle':
      return 'bg-[var(--text-muted)]'
  }
}

function cellAnimClass(state: AgentActivityState, idx: number): string {
  switch (state) {
    case 'thinking':
      return `glyph-pulse ${DELAY_CLASSES[idx] ?? 'glyph-delay-0'}`
    case 'reading':
      return `glyph-scan ${DELAY_CLASSES[(idx * 2) % DELAY_CLASSES.length] ?? 'glyph-delay-0'}`
    case 'analysing': {
      const order = ORBIT_ORDER[idx] ?? 0
      return `glyph-orbit ${DELAY_CLASSES[order] ?? 'glyph-delay-0'}`
    }
    case 'debugging': {
      const order = DIAGONAL_ORDER[idx] ?? 0
      return `glyph-cascade ${DELAY_CLASSES[order] ?? 'glyph-delay-0'}`
    }
    case 'creating': {
      const delayIdx = CORNER_INDICES.has(idx) ? 0 : idx === 4 ? 2 : 4
      return `glyph-pulse ${DELAY_CLASSES[delayIdx] ?? 'glyph-delay-0'}`
    }
    case 'idle':
      return ''
  }
}

export function AgentGlyph({ state, size = 'md' }: AgentGlyphProps) {
  const [hidden, setHidden] = useState(() =>
    typeof document !== 'undefined' ? document.hidden : false,
  )

  useEffect(() => {
    const onVisibility = () => setHidden(document.hidden)
    document.addEventListener('visibilitychange', onVisibility)
    return () => document.removeEventListener('visibilitychange', onVisibility)
  }, [])

  const dim = size === 'sm' ? 'w-4 h-4' : 'w-6 h-6'
  const pausedClass = hidden ? 'glyph-paused' : ''

  return (
    <span
      className={`grid grid-cols-3 gap-0.5 shrink-0 ${dim} ${pausedClass}`}
      aria-hidden="true"
    >
      {DOT_MATRIX.map((idx) => {
        const active = isCellActive(state, idx)
        const color = active ? cellColorClass(state, idx) : 'bg-[var(--dot-inactive)]'
        const anim = active ? cellAnimClass(state, idx) : ''
        return <span key={idx} className={`rounded-xs ${color} ${anim}`} />
      })}
    </span>
  )
}
