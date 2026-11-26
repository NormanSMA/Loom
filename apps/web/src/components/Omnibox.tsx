import { useEffect, useState, type FormEvent } from 'react'
import { ArrowUp, ChevronDown, FolderGit2, Lock, Sparkles } from 'lucide-react'
import { useI18n, type LocaleCode } from '../i18n'
import type { AgentActivityState, LiveEvent, WorkspaceInfo } from '../types/api'

interface OmniboxProps {
  workspaces?: WorkspaceInfo[]
  selectedRepo?: string | null
  onSelectRepo?: (path: string | null) => void
  defaultSensitivity?: 'internal' | 'confidential'
  liveEvent?: LiveEvent | null
  onSubmit: (payload: {
    title: string
    mode: string
    effort: string
    sensitivity: string
    repo_root?: string
  }) => Promise<void>
}

const STATE_STRIP_LABELS: Record<LocaleCode, Record<Exclude<AgentActivityState, 'idle'>, string>> = {
  es: {
    thinking: 'Pensando...',
    reading: 'Leyendo docs...',
    analysing: 'Analizando...',
    debugging: 'Depurando...',
    creating: 'Creando...',
  },
  en: {
    thinking: 'Thinking...',
    reading: 'Reading docs...',
    analysing: 'Analysing...',
    debugging: 'Debugging...',
    creating: 'Creating...',
  },
  'pt-BR': {
    thinking: 'Pensando...',
    reading: 'Lendo docs...',
    analysing: 'Analisando...',
    debugging: 'Depurando...',
    creating: 'Criando...',
  },
}

const DISPATCH_HINT: Record<LocaleCode, string> = {
  es: 'Despachando misión al nodo local',
  en: 'Dispatching mission to local node',
  'pt-BR': 'Enviando missão ao nó local',
}

export function Omnibox({
  workspaces = [],
  selectedRepo = null,
  onSelectRepo,
  defaultSensitivity = 'internal',
  liveEvent = null,
  onSubmit,
}: OmniboxProps) {
  const { locale, t } = useI18n()
  const [title, setTitle] = useState('')
  const [mode, setMode] = useState<'auto_dag' | 'direct' | 'image'>('auto_dag')
  const [effort, setEffort] = useState<'fast' | 'normal' | 'deep_think'>('normal')
  const [sensitivity, setSensitivity] = useState<'internal' | 'confidential'>(defaultSensitivity)
  const [sensitivityTouched, setSensitivityTouched] = useState(false)
  const [busy, setBusy] = useState(false)

  useEffect(() => {
    if (!sensitivityTouched) {
      setSensitivity(defaultSensitivity)
    }
  }, [defaultSensitivity, sensitivityTouched])

  const showStatusStrip = Boolean((liveEvent && liveEvent.state !== 'idle') || busy)
  const activeState: Exclude<AgentActivityState, 'idle'> =
    liveEvent && liveEvent.state !== 'idle' ? liveEvent.state : 'thinking'
  const stripLabel = STATE_STRIP_LABELS[locale][activeState]
  const stripHint =
    liveEvent && liveEvent.state !== 'idle'
      ? [liveEvent.provider_id, liveEvent.target].filter(Boolean).join(' · ')
      : DISPATCH_HINT[locale]

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()
    const clean = title.trim()
    if (!clean || busy) return
    setBusy(true)
    try {
      await onSubmit({
        title: clean,
        mode,
        effort,
        sensitivity,
        ...(selectedRepo ? { repo_root: selectedRepo } : {}),
      })
      setTitle('')
      setSensitivityTouched(false)
      setSensitivity(defaultSensitivity)
    } finally {
      setBusy(false)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="omnibox-glow omnibox-aura rounded-2xl">
      {showStatusStrip && (
        <div
          role="status"
          aria-live="polite"
          className="glass-folder relative z-10 mx-2 -mb-2 flex items-center gap-2.5 rounded-t-2xl border-b-0 px-3.5 pt-2 pb-3.5"
        >
          <span className="orb-thinking shrink-0" aria-hidden="true" />
          <span className="shimmer-text text-xs font-medium tracking-wide">{stripLabel}</span>
          {stripHint && (
            <span className="min-w-0 flex-1 truncate font-mono text-xs text-[var(--text-muted)]">
              {stripHint}
            </span>
          )}
        </div>
      )}

      <div className="loom-elevated relative z-10 rounded-2xl p-3 sm:p-4">
        <div className="flex items-center gap-2.5">
          <Sparkles className="h-4 w-4 shrink-0 text-[var(--accent-blue)]" aria-hidden="true" />
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder={t('omnibox.placeholder')}
            aria-label={t('omnibox.placeholder')}
            className="min-h-12 min-w-0 flex-1 bg-transparent text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none sm:text-base"
          />
          <button
            type="submit"
            disabled={busy || !title.trim()}
            aria-label={t('action.send')}
            className="inline-flex min-h-11 min-w-11 shrink-0 items-center justify-center rounded-xl bg-[var(--accent-blue)] px-3 text-sm font-medium text-white transition-all hover:opacity-90 active:scale-95 disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--accent-blue)]"
          >
            <ArrowUp className="h-4 w-4" aria-hidden="true" />
          </button>
        </div>

        <div className="mt-2.5 flex flex-wrap items-center gap-2 border-t border-[var(--border-subtle)] pt-2.5">
          <div className="relative">
            <select
              aria-label="mode"
              value={mode}
              onChange={(e) => setMode(e.target.value as 'auto_dag' | 'direct' | 'image')}
              className="loom-card min-h-11 appearance-none rounded-xl pr-7 pl-3 text-xs text-[var(--text-primary)] transition-colors hover:border-[var(--border-strong)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
            >
              <option value="auto_dag">{t('omnibox.mode.auto_dag')}</option>
              <option value="direct">{t('omnibox.mode.direct')}</option>
              <option value="image">{t('omnibox.mode.image')}</option>
            </select>
            <ChevronDown
              className="pointer-events-none absolute top-1/2 right-2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-muted)]"
              aria-hidden="true"
            />
          </div>

          <div className="relative">
            <select
              aria-label="effort"
              value={effort}
              onChange={(e) => setEffort(e.target.value as 'fast' | 'normal' | 'deep_think')}
              className="loom-card min-h-11 appearance-none rounded-xl pr-7 pl-3 text-xs text-[var(--text-primary)] transition-colors hover:border-[var(--border-strong)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
            >
              <option value="fast">{t('omnibox.effort.fast')}</option>
              <option value="normal">{t('omnibox.effort.normal')}</option>
              <option value="deep_think">{t('omnibox.effort.deep_think')}</option>
            </select>
            <ChevronDown
              className="pointer-events-none absolute top-1/2 right-2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-muted)]"
              aria-hidden="true"
            />
          </div>

          {workspaces.length > 0 && onSelectRepo && (
            <div className="relative max-w-full">
              <FolderGit2
                className="pointer-events-none absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-[var(--accent-green)]"
                aria-hidden="true"
              />
              <select
                aria-label={t('omnibox.workspaceAuto')}
                value={selectedRepo ?? ''}
                onChange={(e) => onSelectRepo(e.target.value || null)}
                className="loom-card min-h-11 max-w-48 appearance-none truncate rounded-xl pr-7 pl-8 text-xs text-[var(--text-primary)] transition-colors hover:border-[var(--border-strong)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
              >
                <option value="">{t('omnibox.workspaceAuto')}</option>
                {workspaces.map((ws) => (
                  <option key={ws.path} value={ws.path}>
                    {ws.name}
                  </option>
                ))}
              </select>
              <ChevronDown
                className="pointer-events-none absolute top-1/2 right-2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-muted)]"
                aria-hidden="true"
              />
            </div>
          )}

          <button
            type="button"
            aria-pressed={sensitivity === 'confidential'}
            onClick={() => {
              setSensitivityTouched(true)
              setSensitivity((s) => (s === 'internal' ? 'confidential' : 'internal'))
            }}
            className={`inline-flex min-h-11 items-center gap-1.5 rounded-xl border px-3 text-xs transition-colors active:scale-95 focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
              sensitivity === 'confidential'
                ? 'border-[var(--accent-amber)] bg-[var(--accent-amber-soft)] text-[var(--accent-amber)]'
                : 'border-[var(--border-subtle)] bg-[var(--bg-surface)] text-[var(--text-secondary)] hover:border-[var(--border-strong)] hover:text-[var(--text-primary)]'
            }`}
          >
            <Lock className="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
            {t(`omnibox.sens.${sensitivity}`)}
          </button>
        </div>
      </div>
    </form>
  )
}
