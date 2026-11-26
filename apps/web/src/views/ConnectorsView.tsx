import { Link2 } from 'lucide-react'
import { useI18n } from '../i18n'
import type { ClaudeHubBridge, DoctorReport, ProviderManifest } from '../types/api'

interface ConnectorsViewProps {
  doctor: DoctorReport | null
  providers: ProviderManifest[]
  claudehub: ClaudeHubBridge | null
}

export function ConnectorsView({ doctor, providers, claudehub }: ConnectorsViewProps) {
  const { t } = useI18n()

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-[var(--text-primary)]">
          {t('connectors.title')}
        </h1>
        <p className="text-sm text-[var(--text-secondary)]">{t('connectors.subtitle')}</p>
      </div>

      {claudehub && (
        <section className="loom-card flex flex-wrap items-center justify-between gap-3 rounded-2xl p-4">
          <div className="flex items-center gap-3">
            <span className="inline-flex h-11 w-11 items-center justify-center rounded-xl bg-[var(--accent-blue-soft)] text-[var(--accent-blue)]">
              <Link2 className="h-5 w-5" />
            </span>
            <div>
              <h2 className="text-sm font-medium text-[var(--text-primary)]">
                {t('connectors.claudehub')}
              </h2>
              <p className="font-mono text-xs text-[var(--text-muted)]">
                {claudehub.url} {claudehub.version ? `· v${claudehub.version}` : ''}
                {claudehub.level_label ? ` · ${claudehub.level_label}` : ''}
              </p>
            </div>
          </div>
          <span
            className={`rounded-full px-3 py-1 font-mono text-xs ${
              claudehub.connected
                ? 'bg-[var(--accent-green-soft)] text-[var(--accent-green)]'
                : 'bg-[var(--accent-amber-soft)] text-[var(--accent-amber)]'
            }`}
          >
            {claudehub.connected ? t('connectors.connected') : t('connectors.disconnected')}
          </span>
        </section>
      )}

      {doctor && (
        <div className="grid gap-3 sm:grid-cols-2">
          {doctor.checks.map((item) => (
            <article key={item.id} className="loom-card rounded-2xl p-4">
              <div className="flex items-center justify-between gap-2">
                <h3 className="text-sm font-medium text-[var(--text-primary)]">{item.name}</h3>
                <span
                  className={`rounded-md px-2 py-0.5 font-mono text-xs ${
                    item.status === 'ok'
                      ? 'bg-[var(--accent-green-soft)] text-[var(--accent-green)]'
                      : item.status === 'warning'
                        ? 'bg-[var(--accent-amber-soft)] text-[var(--accent-amber)]'
                        : 'bg-[var(--accent-red-soft)] text-[var(--accent-red)]'
                  }`}
                >
                  {item.status.toUpperCase()}
                </span>
              </div>
              <p className="mt-1 font-mono text-xs text-[var(--text-secondary)]">
                {item.version ?? '-'} · {item.latency_ms} ms
              </p>
              {item.warnings.map((w) => (
                <p key={w} className="mt-1 text-xs text-[var(--accent-amber)]">
                  {w}
                </p>
              ))}
            </article>
          ))}
        </div>
      )}

      <section className="loom-card rounded-2xl p-4">
        <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
          {providers.map((p) => (
            <div
              key={p.id}
              className="loom-elevated flex items-center justify-between rounded-xl px-3 py-2.5 text-xs"
            >
              <div>
                <p className="font-medium text-[var(--text-primary)]">{p.name}</p>
                <p className="font-mono text-[var(--text-muted)]">
                  {p.tier} · {p.privacy}
                </p>
              </div>
              <span
                className={`h-2 w-2 rounded-full ${
                  p.enabled ? 'bg-[var(--accent-green)]' : 'bg-[var(--dot-inactive)]'
                }`}
              />
            </div>
          ))}
        </div>
      </section>
    </div>
  )
}
