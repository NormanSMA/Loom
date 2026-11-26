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
  const target = event?.target ?? t('status.nodeReady')
  const detail = event?.detail ?? t('status.online')
  const provider = event?.provider_id ?? 'loom'

  return (
    <div className="flex justify-center px-3 pt-2.5">
      <button
        type="button"
        onClick={() => setExpanded((prev) => !prev)}
        aria-expanded={expanded}
        className="loom-glass-island min-h-11 max-w-full rounded-2xl px-3.5 py-2 text-left transition-transform duration-150 active:scale-[0.99] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] sm:px-4"
      >
        <div className="flex items-center gap-3">
          <AgentGlyph state={state} size="sm" />
          <div className="min-w-0">
            <p className="truncate font-mono text-[11px] text-[var(--text-secondary)]">
              {target}
            </p>
            <p className="truncate text-xs font-medium text-[var(--text-primary)] sm:text-sm">
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
