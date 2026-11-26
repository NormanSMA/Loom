import { useI18n, type LocaleCode } from '../i18n'
import type { TelemetrySnapshot } from '../types/api'
import { DotHeatmap } from '../components/DotHeatmap'
import { DotMatrixEqualizer, DotMatrixSparkline } from '../components/DotMatrixWidgets'
import { LedBarCard } from '../components/LedBarCard'

interface TelemetryViewProps {
  telemetry: TelemetrySnapshot | null
}

interface TelemetryMatrixCopy {
  equalizerTitle: string
  equalizerSubtitle: string
  equalizerUnit: string
  sparklineTitle: string
  sparklineSubtitle: string
  avgPerDay: string
  peakDay: string
  activeDays: string
  totalCalls: string
}

const TELEMETRY_MATRIX_COPY: Record<LocaleCode, TelemetryMatrixCopy> = {
  es: {
    equalizerTitle: 'Pulso de los últimos 14 días',
    equalizerSubtitle: 'Selecciona o enfoca una columna para ver el día',
    equalizerUnit: 'regs',
    sparklineTitle: 'Curva diaria y registros',
    sparklineSubtitle: 'Tendencia reciente de actividad y uso por proveedor',
    avgPerDay: 'Promedio/día',
    peakDay: 'Pico máximo',
    activeDays: 'Días activos',
    totalCalls: 'Total regs',
  },
  en: {
    equalizerTitle: '14-day activity pulse',
    equalizerSubtitle: 'Select or focus a column to inspect the day',
    equalizerUnit: 'calls',
    sparklineTitle: 'Daily curve and calls',
    sparklineSubtitle: 'Recent activity trend and provider call totals',
    avgPerDay: 'Avg / day',
    peakDay: 'Peak day',
    activeDays: 'Active days',
    totalCalls: 'Total calls',
  },
  'pt-BR': {
    equalizerTitle: 'Pulso dos últimos 14 dias',
    equalizerSubtitle: 'Selecione ou foque uma coluna para ver o dia',
    equalizerUnit: 'regs',
    sparklineTitle: 'Curva diária e registros',
    sparklineSubtitle: 'Tendência recente de atividade e uso por provedor',
    avgPerDay: 'Média/dia',
    peakDay: 'Pico máximo',
    activeDays: 'Dias ativos',
    totalCalls: 'Total regs',
  },
}

function formatCompactTokens(n: number): string {
  if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return String(n)
}

export function TelemetryView({ telemetry }: TelemetryViewProps) {
  const { locale, t } = useI18n()
  if (!telemetry) return null

  const copy = TELEMETRY_MATRIX_COPY[locale]
  const tokenRatio =
    telemetry.total_tokens <= 0
      ? 0
      : Math.min(1, Math.log10(telemetry.total_tokens + 1) / 10)

  const totalProviderCalls = telemetry.by_provider.reduce((sum, row) => sum + row.calls, 0)

  return (
    <div className="min-w-0 space-y-5">
      <div>
        <h1 className="text-xl font-semibold text-[var(--text-primary)]">
          {t('telemetry.title')}
        </h1>
        <p className="text-sm text-[var(--text-secondary)]">{t('telemetry.subtitle')}</p>
      </div>

      <div className="grid grid-cols-2 gap-3 xl:grid-cols-4">
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

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <DotMatrixEqualizer
          points={telemetry.heatmap}
          title={copy.equalizerTitle}
          subtitle={copy.equalizerSubtitle}
          unitLabel={copy.equalizerUnit}
        />
        <DotMatrixSparkline
          points={telemetry.heatmap}
          title={copy.sparklineTitle}
          subtitle={copy.sparklineSubtitle}
          totalOverride={totalProviderCalls > 0 ? totalProviderCalls : undefined}
          labels={{
            avgPerDay: copy.avgPerDay,
            peakDay: copy.peakDay,
            activeDays: copy.activeDays,
            totalCalls: copy.totalCalls,
          }}
        />
      </div>

      <DotHeatmap points={telemetry.heatmap} title={t('telemetry.heatmap')} />

      <div className="grid gap-4 lg:grid-cols-2">
        <section className="loom-card min-w-0 overflow-x-auto rounded-2xl p-4">
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

        <section className="loom-card min-w-0 rounded-2xl p-4">
          <h2 className="mb-3 text-sm font-medium text-[var(--text-primary)]">
            {t('telemetry.logs')}
          </h2>
          {telemetry.logs.length === 0 ? (
            <p className="text-xs text-[var(--text-muted)]">{t('telemetry.emptyLogs')}</p>
          ) : (
            <div className="space-y-2">
              {telemetry.logs.map((log, i) => (
                <div key={`${log.ts}-${i}`} className="loom-elevated min-w-0 rounded-xl px-3 py-2 text-xs">
                  <span className="font-mono text-[var(--accent-blue)]">[{log.provider_id}] </span>
                  <span className="break-words text-[var(--text-secondary)]">{log.message}</span>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>
    </div>
  )
}
