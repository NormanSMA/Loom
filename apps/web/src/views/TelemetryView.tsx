import { useI18n } from '../i18n'
import type { TelemetrySnapshot } from '../types/api'
import { DotHeatmap } from '../components/DotHeatmap'
import { LedBarCard } from '../components/LedBarCard'

interface TelemetryViewProps {
  telemetry: TelemetrySnapshot | null
}

function formatCompactTokens(n: number): string {
  if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return String(n)
}

export function TelemetryView({ telemetry }: TelemetryViewProps) {
  const { t } = useI18n()
  if (!telemetry) return null

  const tokenRatio =
    telemetry.total_tokens <= 0
      ? 0
      : Math.min(1, Math.log10(telemetry.total_tokens + 1) / 10)

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold text-[var(--text-primary)]">
          {t('telemetry.title')}
        </h1>
        <p className="text-sm text-[var(--text-secondary)]">{t('telemetry.subtitle')}</p>
      </div>

      <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <LedBarCard
          label={t('telemetry.tokens')}
          value={formatCompactTokens(telemetry.total_tokens)}
          hint={telemetry.total_tokens > 0 ? telemetry.total_tokens.toLocaleString() : undefined}
          fillRatio={tokenRatio}
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
          value={`${telemetry.gate_pass_rate.toFixed(0)}%`}
          fillRatio={telemetry.gate_pass_rate / 100}
          tone="green"
        />
        <LedBarCard
          label={t('telemetry.latency')}
          value={`${telemetry.avg_latency_ms} ms`}
          fillRatio={telemetry.avg_latency_ms === 0 ? 0 : Math.min(1, telemetry.avg_latency_ms / 1500)}
          tone="amber"
        />
      </div>

      <DotHeatmap points={telemetry.heatmap} title={t('telemetry.heatmap')} />

      <div className="grid gap-4 lg:grid-cols-2">
        <section className="loom-card overflow-x-auto rounded-2xl p-4">
          {telemetry.by_provider.length === 0 ? (
            <p className="text-xs text-[var(--text-muted)]">{t('telemetry.emptyProviders')}</p>
          ) : (
            <table className="w-full text-left text-xs">
              <thead className="border-b border-[var(--border-subtle)] text-[var(--text-muted)]">
                <tr>
                  <th className="py-2 pr-3">Provider</th>
                  <th className="py-2 pr-3">Regs</th>
                  <th className="py-2 pr-3">OK %</th>
                  <th className="py-2">429/403</th>
                </tr>
              </thead>
              <tbody>
                {telemetry.by_provider.map((row) => (
                  <tr key={row.provider_id} className="border-b border-[var(--border-subtle)]">
                    <td className="py-2 pr-3 font-medium text-[var(--text-primary)]">{row.name}</td>
                    <td className="py-2 pr-3 font-mono text-[var(--text-secondary)]">
                      {row.calls.toLocaleString()}
                    </td>
                    <td className="py-2 pr-3 font-mono text-[var(--accent-green)]">
                      {row.success_rate.toFixed(0)}%
                    </td>
                    <td className="py-2 font-mono text-[var(--text-muted)]">
                      {row.errors_429}/{row.errors_403}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </section>

        <section className="loom-card rounded-2xl p-4">
          <h2 className="mb-3 text-sm font-medium text-[var(--text-primary)]">
            {t('telemetry.logs')}
          </h2>
          {telemetry.logs.length === 0 ? (
            <p className="text-xs text-[var(--text-muted)]">{t('telemetry.emptyLogs')}</p>
          ) : (
            <div className="space-y-2">
              {telemetry.logs.map((log, i) => (
                <div key={`${log.ts}-${i}`} className="loom-elevated rounded-xl px-3 py-2 text-xs">
                  <span className="font-mono text-[var(--accent-blue)]">[{log.provider_id}] </span>
                  <span className="text-[var(--text-secondary)]">{log.message}</span>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>
    </div>
  )
}
