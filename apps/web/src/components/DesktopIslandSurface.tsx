import { useState } from 'react'
import { ChevronDown, ChevronUp } from 'lucide-react'
import { useLoomState } from '../hooks/useLoomState'
import { useI18n } from '../i18n'
import { AgentGlyph } from './AgentGlyph'

export function DesktopIslandSurface() {
  const { t } = useI18n()
  const loom = useLoomState()
  const [expanded, setExpanded] = useState(false)

  const event = loom.liveEvent
  const state = loom.isOffline ? 'idle' : (event?.state ?? 'idle')
  const stateLabel = t(`status.${state}`)
  const target = event?.target ?? 'loomd · 127.0.0.1:4320'
  const detail = event?.detail ?? (loom.isOffline ? t('status.offline') : t('status.online'))
  const provider = event?.provider_id ?? 'loom'

  return (
    <div data-tauri-drag-region className="flex w-full justify-center px-2">
      <button
        type="button"
        onClick={() => setExpanded((prev) => !prev)}
        aria-expanded={expanded}
        className="notch-shell notch-no-drag min-h-11 w-full max-w-xs px-4 py-2 text-left focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
      >
        <p className="truncate font-mono text-xs text-[var(--text-secondary)]">{target}</p>
        <div className="mt-1 flex items-center justify-between gap-2">
          <div className="flex min-w-0 items-center gap-2">
            <AgentGlyph state={state} size="sm" />
            <span className="shimmer-text truncate text-xs font-medium">{stateLabel}</span>
          </div>
          {expanded ? (
            <ChevronUp className="h-4 w-4 shrink-0 text-[var(--text-muted)]" aria-hidden="true" />
          ) : (
            <ChevronDown className="h-4 w-4 shrink-0 text-[var(--text-muted)]" aria-hidden="true" />
          )}
        </div>
        <div className="mt-1 flex items-center">
          <span className="inline-flex items-center rounded-full border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-2 py-0.5 font-mono text-xs text-[var(--accent-blue)]">
            {provider}
          </span>
        </div>
        {expanded && (
          <p className="mt-2 border-t border-[var(--border-subtle)] pt-1.5 text-xs text-[var(--text-secondary)]">
            {detail}
          </p>
        )}
      </button>
    </div>
  )
}
