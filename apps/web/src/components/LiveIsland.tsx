import { useState } from 'react'
import { ChevronDown, ChevronUp } from 'lucide-react'
import { useI18n } from '../i18n'
import type { LiveEvent } from '../types/api'
import { AgentGlyph } from './AgentGlyph'

interface LiveIslandProps {
  event: LiveEvent | null
}

export function LiveIsland({ event }: LiveIslandProps) {
  const { t } = useI18n()
  const [expanded, setExpanded] = useState(false)

  const state = event?.state ?? 'idle'
  const stateLabel = t(`status.${state}`)
  const target = event?.target ?? 'loomd · 127.0.0.1:4320'
  const detail = event?.detail ?? t('status.online')
  const provider = event?.provider_id ?? 'loom'

  return (
    <div className="flex justify-center px-3 pt-2">
      <button
        type="button"
        onClick={() => setExpanded((prev) => !prev)}
        aria-expanded={expanded}
        className="loom-glass-island min-h-11 rounded-2xl px-4 py-2 text-left transition-transform duration-150 active:scale-[0.99] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
      >
        <div className="flex items-center gap-3">
          <AgentGlyph state={state} size="sm" />
          <div className="min-w-0">
            <p className="truncate text-xs font-mono text-[var(--text-secondary)]">
              {target}
            </p>
            <p className="truncate text-sm font-medium text-[var(--text-primary)]">
              {stateLabel} · <span className="text-[var(--accent-blue)]">{provider}</span>
            </p>
          </div>
          {expanded ? (
            <ChevronUp className="h-4 w-4 shrink-0 text-[var(--text-muted)]" />
          ) : (
            <ChevronDown className="h-4 w-4 shrink-0 text-[var(--text-muted)]" />
          )}
        </div>
        {expanded && (
          <p className="mt-2 border-t border-[var(--border-subtle)] pt-2 text-xs text-[var(--text-secondary)]">
            {detail}
          </p>
        )}
      </button>
    </div>
  )
}
