import { motion } from 'motion/react'
import { useI18n } from '../i18n'
import type { MissionView } from '../types/api'
import { AgentGlyph } from '../components/AgentGlyph'
import { Omnibox } from '../components/Omnibox'

interface MissionsViewProps {
  missions: MissionView[]
  onCreateMission: (payload: {
    title: string
    mode: string
    effort: string
    sensitivity: string
  }) => Promise<void>
}

export function MissionsView({ missions, onCreateMission }: MissionsViewProps) {
  const { t } = useI18n()

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-[var(--text-primary)]">
          {t('missions.title')}
        </h1>
        <p className="text-sm text-[var(--text-secondary)]">{t('missions.subtitle')}</p>
      </div>

      <Omnibox onSubmit={onCreateMission} />

      {missions.length === 0 ? (
        <div className="loom-card rounded-2xl p-6 text-center text-sm text-[var(--text-muted)]">
          {t('missions.empty')}
        </div>
      ) : (
        <div className="grid gap-3">
          {missions.map((m, idx) => (
            <motion.article
              key={m.id}
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.18, delay: Math.min(idx * 0.04, 0.2) }}
              className="loom-card flex flex-col justify-between gap-3 rounded-2xl p-4 sm:flex-row sm:items-center"
            >
              <div className="flex items-start gap-3">
                <div className="pt-0.5">
                  <AgentGlyph state={m.state} />
                </div>
                <div className="min-w-0">
                  <h2 className="text-sm font-medium text-[var(--text-primary)]">{m.title}</h2>
                  <p className="mt-1 font-mono text-xs text-[var(--text-muted)]">
                    {m.worktree} · {m.mode} · {m.effort}
                  </p>
                </div>
              </div>

              <div className="flex flex-wrap items-center gap-2">
                <span className="loom-elevated rounded-lg px-2.5 py-1 font-mono text-xs text-[var(--accent-blue)]">
                  {m.provider_id}
                </span>
                <span className="loom-elevated rounded-lg px-2.5 py-1 font-mono text-xs text-[var(--text-secondary)]">
                  {t(`status.${m.state}`)}
                </span>
              </div>
            </motion.article>
          ))}
        </div>
      )}
    </div>
  )
}
