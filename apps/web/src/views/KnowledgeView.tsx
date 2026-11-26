import { useMemo, useState } from 'react'
import { Search } from 'lucide-react'
import { useI18n } from '../i18n'
import type { SkillEntry } from '../types/api'

interface KnowledgeViewProps {
  skills: SkillEntry[]
}

export function KnowledgeView({ skills }: KnowledgeViewProps) {
  const { t } = useI18n()
  const [query, setQuery] = useState('')
  const [sourceFilter, setSourceFilter] = useState<string>('all')
  const [expanded, setExpanded] = useState<Record<string, boolean>>({})

  const sources = useMemo(
    () => Array.from(new Set(skills.map((s) => s.source))).sort(),
    [skills],
  )

  const q = query.trim().toLowerCase()
  const filtered = skills.filter((s) => {
    const matchesSource = sourceFilter === 'all' || s.source === sourceFilter
    const matchesQuery =
      !q || s.name.toLowerCase().includes(q) || s.description.toLowerCase().includes(q)
    return matchesSource && matchesQuery
  })

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <div>
          <h1 className="text-xl font-semibold text-[var(--text-primary)]">
            {t('knowledge.title')}
          </h1>
          <p className="text-sm text-[var(--text-secondary)]">{t('knowledge.subtitle')}</p>
        </div>
        <span className="loom-elevated rounded-full px-3 py-1 font-mono text-xs text-[var(--text-secondary)]">
          {filtered.length} {t('knowledge.count')}
        </span>
      </div>

      <div className="loom-elevated flex min-h-11 items-center gap-2 rounded-xl px-3">
        <Search className="h-4 w-4 shrink-0 text-[var(--text-muted)]" aria-hidden="true" />
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={t('knowledge.search')}
          aria-label={t('knowledge.search')}
          className="min-h-11 min-w-0 flex-1 bg-transparent text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none"
        />
      </div>

      {sources.length > 1 && (
        <div className="flex gap-2 overflow-x-auto pb-1">
          <button
            type="button"
            onClick={() => setSourceFilter('all')}
            aria-pressed={sourceFilter === 'all'}
            className={`min-h-11 shrink-0 rounded-full border px-3.5 text-xs transition-colors focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
              sourceFilter === 'all'
                ? 'border-[var(--accent-blue)] bg-[var(--accent-blue-soft)] font-medium text-[var(--accent-blue)]'
                : 'border-[var(--border-subtle)] bg-[var(--bg-surface)] text-[var(--text-secondary)]'
            }`}
          >
            {t('knowledge.allSources')}
          </button>
          {sources.map((src) => (
            <button
              key={src}
              type="button"
              onClick={() => setSourceFilter(src)}
              aria-pressed={sourceFilter === src}
              className={`min-h-11 shrink-0 rounded-full border px-3.5 font-mono text-xs transition-colors focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
                sourceFilter === src
                  ? 'border-[var(--accent-blue)] bg-[var(--accent-blue-soft)] font-medium text-[var(--accent-blue)]'
                  : 'border-[var(--border-subtle)] bg-[var(--bg-surface)] text-[var(--text-secondary)]'
              }`}
            >
              {src}
            </button>
          ))}
        </div>
      )}

      <div className="grid gap-3 sm:grid-cols-2">
        {filtered.map((skill) => {
          const id = `${skill.source}-${skill.name}`
          const isOpen = Boolean(expanded[id])
          return (
            <article key={id} className="loom-card rounded-2xl p-4">
              <div className="flex items-start justify-between gap-2">
                <h2 className="min-w-0 break-words font-mono text-sm font-medium text-[var(--text-primary)]">
                  {skill.name}
                </h2>
                <span className="loom-elevated shrink-0 rounded-md px-2 py-0.5 font-mono text-[11px] text-[var(--accent-blue)]">
                  {skill.source}
                </span>
              </div>
              <p
                className={`mt-1.5 text-xs text-[var(--text-secondary)] ${
                  isOpen ? '' : 'line-clamp-2'
                }`}
              >
                {skill.description}
              </p>
              {skill.description.length > 90 && (
                <button
                  type="button"
                  aria-expanded={isOpen}
                  onClick={() => setExpanded((prev) => ({ ...prev, [id]: !prev[id] }))}
                  className="mt-2 inline-flex min-h-11 items-center text-xs font-medium text-[var(--accent-blue)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
                >
                  {isOpen ? t('knowledge.collapse') : t('knowledge.expand')}
                </button>
              )}
            </article>
          )
        })}
      </div>
    </div>
  )
}
