import { useEffect, useState } from 'react'
import type { AgentActivityState } from '../types/api'

interface AgentGlyphProps {
  state: AgentActivityState
  size?: 'sm' | 'md'
}

const DOT_MATRIX = [0, 1, 2, 3, 4, 5, 6, 7, 8]

export function AgentGlyph({ state, size = 'md' }: AgentGlyphProps) {
  const [hidden, setHidden] = useState(false)

  useEffect(() => {
    const onVisibility = () => setHidden(document.hidden)
    document.addEventListener('visibilitychange', onVisibility)
    return () => document.removeEventListener('visibilitychange', onVisibility)
  }, [])

  const dim = size === 'sm' ? 'w-4 h-4' : 'w-6 h-6'
  const pausedClass = hidden ? 'glyph-paused' : ''

  const colorClass = (() => {
    switch (state) {
      case 'thinking':
      case 'creating':
        return 'bg-[var(--accent-green)]'
      case 'reading':
        return 'bg-[var(--accent-red)]'
      case 'analysing':
      case 'debugging':
        return 'bg-[var(--accent-blue)]'
      case 'idle':
        return 'bg-[var(--text-muted)]'
    }
  })()

  const animClass = (() => {
    switch (state) {
      case 'thinking':
        return 'glyph-pulse'
      case 'reading':
        return 'glyph-scan'
      case 'analysing':
      case 'debugging':
      case 'creating':
        return 'glyph-cascade'
      case 'idle':
        return ''
    }
  })()

  return (
    <span
      className={`grid grid-cols-3 gap-0.5 shrink-0 ${dim} ${pausedClass}`}
      aria-hidden="true"
    >
      {DOT_MATRIX.map((idx) => {
        const active =
          state === 'idle'
            ? idx === 4
            : state === 'reading'
              ? idx % 2 === 0
              : state === 'debugging'
                ? idx >= 2 && idx <= 7
                : true
        return (
          <span
            key={idx}
            className={`rounded-[1px] ${active ? colorClass : 'bg-[var(--dot-inactive)]'} ${active ? animClass : ''}`}
          />
        )
      })}
    </span>
  )
}
