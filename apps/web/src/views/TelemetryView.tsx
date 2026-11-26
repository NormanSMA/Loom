import { useI18n } from '../i18n'
import type { TelemetrySnapshot } from '../types/api'
import { DotHeatmap } from '../components/DotHeatmap'
import { LedBarCard } from '../components/LedBarCard'

interface TelemetryViewProps {
  telemetry: TelemetrySnapshot | null
}

export function TelemetryView({ telemetry }: TelemetryViewProps) {
  const { t } = useI18n()
  if (!telemetry) return null

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-[var(--text-primary)]">
          {t('telemetry.title')}
        </h1>
        <p className="text-sm text-[var(--text-secondary)]">{t('telemetry.subtitle')}</p>
      </div>

      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <LedBarCard
          label={t('telemetry.tokens')}
          value={telemetry.total_tokens.toLocaleString()}
          fillRatio={0.7}
          tone="blue"
        />
        <LedBarCard
          label={t('telemetry.agents')}
          value={String(telemetry.active_agents)}
          fillRatio={Math.min(1, telemetry.active_agents / 6)}
          tone="green"
        />
        <LedBarCard
          label={t('telemetry.gates')}
          value={`${telemetry.gate_pass_rate.toFixed(1)}%`}
          fillRatio={telemetry.gate_pass_rate / 100}
          tone="green"
        />
        <LedBarCard
          label={t('telemetry.latency')}
          value={`${telemetry.avg_latency_ms} ms`}
          fillRatio={Math.min(1, telemetry.avg_latency_ms / 1500)}
          tone="amber"
        />
      </div>

      <DotHeatmap points={telemetry.heatmap} title={t('telemetry.heatmap')} />

      <div className="grid gap-4 lg:grid-cols-2">
        <section className="loom-card overflow-x-auto rounded-2xl p-4">
          <table className="w-full text-left text-xs">
            <thead className="border-b border-[var(--border-subtle)] text-[var(--text-muted)]">
              <tr>
                <th className="py-2 pr-3">Provider</th>
                <th className="py-2 pr-3">OK %</th>
                <th className="py-2 pr-3">p95</th>
                <th className="py-2">429/403</th>
              </tr>
            </thead>
            <tbody>
              {telemetry.by_provider.map((row) => (
                <tr key={row.provider_id} className="border-b border-[var(--border-subtle)]">
                  <td className="py-2 pr-3 font-medium text-[var(--text-primary)]">{row.name}</td>
                  <td className="py-2 pr-3 font-mono text-[var(--accent-green)]">
                    {row.success_rate.toFixed(1)}%
                  </td>
                  <td className="py-2 pr-3 font-mono text-[var(--text-secondary)]">
                    {row.p95_ms} ms
                  </td>
                  <td className="py-2 font-mono text-[var(--text-muted)]">
                    {row.errors_429}/{row.errors_403}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>

        <section className="loom-card rounded-2xl p-4">
          <h2 className="mb-3 text-sm font-medium text-[var(--text-primary)]">
            {t('telemetry.logs')}
          </h2>
          <div className="space-y-2">
            {telemetry.logs.map((log, i) => (
              <div key={`${log.ts}-${i}`} className="loom-elevated rounded-xl px-3 py-2 text-xs">
                <span className="font-mono text-[var(--accent-blue)]">[{log.provider_id}] </span>
                <span className="text-[var(--text-secondary)]">{log.message}</span>
              </div>
            ))}
          </div>
        </section>
      </div>
    </div>
  )
}
