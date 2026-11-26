import { useState, type FormEvent } from 'react'
import { ArrowUp, Lock, Sparkles } from 'lucide-react'
import { useI18n } from '../i18n'

interface OmniboxProps {
  onSubmit: (payload: {
    title: string
    mode: string
    effort: string
    sensitivity: string
  }) => Promise<void>
}

export function Omnibox({ onSubmit }: OmniboxProps) {
  const { t } = useI18n()
  const [title, setTitle] = useState('')
  const [mode, setMode] = useState<'auto_dag' | 'direct' | 'image'>('auto_dag')
  const [effort, setEffort] = useState<'fast' | 'normal' | 'deep_think'>('normal')
  const [sensitivity, setSensitivity] = useState<'internal' | 'confidential'>('internal')
  const [busy, setBusy] = useState(false)

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()
    const clean = title.trim()
    if (!clean || busy) return
    setBusy(true)
    try {
      await onSubmit({ title: clean, mode, effort, sensitivity })
      setTitle('')
    } finally {
      setBusy(false)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="omnibox-glow rounded-2xl">
      <div className="loom-elevated relative z-10 rounded-2xl p-3 sm:p-4">
        <div className="flex items-center gap-2">
          <Sparkles className="h-4 w-4 shrink-0 text-[var(--accent-blue)]" aria-hidden="true" />
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder={t('omnibox.placeholder')}
            aria-label={t('omnibox.placeholder')}
            className="min-h-11 w-full bg-transparent text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none"
          />
          <button
            type="submit"
            disabled={busy || !title.trim()}
            aria-label={t('action.send')}
            className="inline-flex min-h-11 min-w-11 items-center justify-center rounded-xl bg-[var(--accent-blue)] px-3 text-sm font-medium text-white transition-opacity disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--accent-blue)]"
          >
            <ArrowUp className="h-4 w-4" />
          </button>
        </div>

        <div className="mt-2 flex flex-wrap items-center gap-2 border-t border-[var(--border-subtle)] pt-2">
          <select
            aria-label="mode"
            value={mode}
            onChange={(e) => setMode(e.target.value as 'auto_dag' | 'direct' | 'image')}
            className="loom-card min-h-11 rounded-lg px-2.5 text-xs text-[var(--text-primary)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
          >
            <option value="auto_dag">{t('omnibox.mode.auto_dag')}</option>
            <option value="direct">{t('omnibox.mode.direct')}</option>
            <option value="image">{t('omnibox.mode.image')}</option>
          </select>

          <select
            aria-label="effort"
            value={effort}
            onChange={(e) => setEffort(e.target.value as 'fast' | 'normal' | 'deep_think')}
            className="loom-card min-h-11 rounded-lg px-2.5 text-xs text-[var(--text-primary)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
          >
            <option value="fast">{t('omnibox.effort.fast')}</option>
            <option value="normal">{t('omnibox.effort.normal')}</option>
            <option value="deep_think">{t('omnibox.effort.deep_think')}</option>
          </select>

          <button
            type="button"
            onClick={() =>
              setSensitivity((s) => (s === 'internal' ? 'confidential' : 'internal'))
            }
            className={`inline-flex min-h-11 items-center gap-1.5 rounded-lg border px-2.5 text-xs transition-colors focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
              sensitivity === 'confidential'
                ? 'border-[var(--accent-amber)] bg-[var(--accent-amber-soft)] text-[var(--accent-amber)]'
                : 'border-[var(--border-subtle)] bg-[var(--bg-surface)] text-[var(--text-secondary)]'
            }`}
          >
            <Lock className="h-3.5 w-3.5" aria-hidden="true" />
            {t(`omnibox.sens.${sensitivity}`)}
          </button>
        </div>
      </div>
    </form>
  )
}
