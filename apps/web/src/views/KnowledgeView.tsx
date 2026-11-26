import { useState } from 'react'
import { Search } from 'lucide-react'
import { useI18n } from '../i18n'
import type { SkillEntry } from '../types/api'

interface KnowledgeViewProps {
  skills: SkillEntry[]
}

export function KnowledgeView({ skills }: KnowledgeViewProps) {
  const { t } = useI18n()
  const [query, setQuery] = useState('')

  const q = query.trim().toLowerCase()
  const filtered = skills.filter(
    (s) =>
      !q || s.name.toLowerCase().includes(q) || s.description.toLowerCase().includes(q),
  )

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold text-[var(--text-primary)]">
          {t('knowledge.title')}
        </h1>
        <p className="text-sm text-[var(--text-secondary)]">{t('knowledge.subtitle')}</p>
      </div>

      <div className="loom-elevated flex min-h-11 items-center gap-2 rounded-xl px-3">
        <Search className="h-4 w-4 text-[var(--text-muted)]" aria-hidden="true" />
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={t('knowledge.search')}
          aria-label={t('knowledge.search')}
          className="min-h-11 w-full bg-transparent text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none"
        />
      </div>

      <div className="grid gap-3 sm:grid-cols-2">
        {filtered.map((skill) => (
          <article key={`${skill.source}-${skill.name}`} className="loom-card rounded-2xl p-4">
            <div className="flex items-center justify-between gap-2">
              <h2 className="font-mono text-sm font-medium text-[var(--text-primary)]">
                {skill.name}
              </h2>
              <span className="loom-elevated rounded-md px-2 py-0.5 font-mono text-[11px] text-[var(--accent-blue)]">
                {skill.source}
              </span>
            </div>
            <p className="mt-1.5 text-xs text-[var(--text-secondary)]">{skill.description}</p>
          </article>
        ))}
      </div>
    </div>
  )
}
