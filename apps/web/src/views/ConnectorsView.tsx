import { BrandIcon } from '../components/BrandIcon'
import { DotMatrixRing, type MatrixTone } from '../components/DotMatrixWidgets'
import { useI18n, type LocaleCode } from '../i18n'
import type { ClaudeHubBridge, DoctorReport, ProviderManifest } from '../types/api'

interface ConnectorsViewProps {
  doctor: DoctorReport | null
  providers: ProviderManifest[]
  claudehub: ClaudeHubBridge | null
}

interface ConnectorsCopy {
  summaryTitle: string
  healthLabel: string
  healthyChecks: string
  alertsOffline: string
  enabledProviders: string
  avgLatency: string
  noChecks: string
}

const CONNECTORS_COPY: Record<LocaleCode, ConnectorsCopy> = {
  es: {
    summaryTitle: 'Estado del ecosistema',
    healthLabel: 'Salud',
    healthyChecks: 'Checks sanos',
    alertsOffline: 'Alertas / off',
    enabledProviders: 'Proveedores',
    avgLatency: 'Latencia media',
    noChecks: 'Sin diagnóstico aún',
  },
  en: {
    summaryTitle: 'Ecosystem health',
    healthLabel: 'Health',
    healthyChecks: 'Healthy checks',
    alertsOffline: 'Alerts / off',
    enabledProviders: 'Providers',
    avgLatency: 'Avg latency',
    noChecks: 'No diagnostics yet',
  },
  'pt-BR': {
    summaryTitle: 'Saúde do ecossistema',
    healthLabel: 'Saúde',
    healthyChecks: 'Checks saudáveis',
    alertsOffline: 'Alertas / off',
    enabledProviders: 'Provedores',
    avgLatency: 'Latência média',
    noChecks: 'Sem diagnóstico ainda',
  },
}

export function ConnectorsView({ doctor, providers, claudehub }: ConnectorsViewProps) {
  const { locale, t } = useI18n()
  const copy = CONNECTORS_COPY[locale]

  const totalChecks = doctor ? doctor.checks.length : 0
  const okChecks = doctor ? doctor.summary.ok : 0
  const issueChecks = doctor ? doctor.summary.warning + doctor.summary.unavailable : 0
  const enabledProviders = providers.filter((p) => p.enabled).length

  const avgLatencyMs =
    doctor && doctor.checks.length > 0
      ? Math.round(
          doctor.checks.reduce((acc, item) => acc + item.latency_ms, 0) / doctor.checks.length,
        )
      : 0

  const healthRatio =
    totalChecks > 0
      ? okChecks / totalChecks
      : providers.length > 0
        ? enabledProviders / providers.length
        : 0

  const ringTone: MatrixTone =
    issueChecks === 0 && totalChecks > 0
      ? 'green'
      : healthRatio >= 0.5
        ? 'amber'
        : 'blue'

  const centerPct = `${Math.round(healthRatio * 100)}%`
  const caption = doctor ? `${doctor.os} · ${doctor.arch}` : copy.noChecks

  return (
    <div className="min-w-0 space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-[var(--text-primary)]">
          {t('connectors.title')}
        </h1>
        <p className="text-sm text-[var(--text-secondary)]">{t('connectors.subtitle')}</p>
      </div>

      <div
        className={`grid gap-4 ${
          claudehub ? 'lg:grid-cols-[minmax(0,1.25fr)_minmax(0,1fr)]' : 'grid-cols-1'
        }`}
      >
        <DotMatrixRing
          ratio={healthRatio}
          valueLabel={copy.healthLabel}
          centerValue={centerPct}
          title={copy.summaryTitle}
          caption={caption}
          tone={ringTone}
          stats={[
            {
              label: copy.healthyChecks,
              value: `${okChecks}/${totalChecks}`,
              tone: 'green',
            },
            {
              label: copy.alertsOffline,
              value: String(issueChecks),
              tone: 'amber',
            },
            {
              label: copy.enabledProviders,
              value: `${enabledProviders}/${providers.length}`,
              tone: 'blue',
            },
            {
              label: copy.avgLatency,
              value: `${avgLatencyMs} ms`,
              tone: 'blue',
            },
          ]}
        />

        {claudehub && (
          <section className="loom-card flex min-w-0 flex-col justify-between gap-4 rounded-2xl p-4">
            <div className="flex min-w-0 items-start justify-between gap-3">
              <div className="flex min-w-0 items-center gap-3">
                <span className="inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-[var(--accent-blue-soft)] text-[var(--accent-blue)]">
                  <BrandIcon id="claudehub" className="h-5 w-5" />
                </span>
                <div className="min-w-0">
                  <h2 className="truncate text-sm font-medium text-[var(--text-primary)]">
                    {t('connectors.claudehub')}
                  </h2>
                  <p className="truncate font-mono text-xs text-[var(--text-muted)]">
                    {claudehub.url}
                  </p>
                </div>
              </div>
              <span
                className={`shrink-0 rounded-full px-3 py-1 font-mono text-xs ${
                  claudehub.connected
                    ? 'bg-[var(--accent-green-soft)] text-[var(--accent-green)]'
                    : 'bg-[var(--accent-amber-soft)] text-[var(--accent-amber)]'
                }`}
              >
                {claudehub.connected ? t('connectors.connected') : t('connectors.disconnected')}
              </span>
            </div>

            <div className="flex flex-wrap items-center gap-2 border-t border-[var(--border-subtle)] pt-3 font-mono text-xs text-[var(--text-secondary)]">
              {claudehub.version && (
                <span className="loom-elevated rounded-lg px-2.5 py-1">
                  v{claudehub.version}
                </span>
              )}
              {claudehub.level_label && (
                <span className="loom-elevated rounded-lg px-2.5 py-1 text-[var(--accent-blue)]">
                  {claudehub.level_label}
                </span>
              )}
              {typeof claudehub.today_fresh === 'number' && (
                <span className="loom-elevated rounded-lg px-2.5 py-1 text-[var(--accent-green)] tabular-nums">
                  {claudehub.today_fresh} today
                </span>
              )}
            </div>
          </section>
        )}
      </div>

      {doctor && (
        <div className="grid gap-3 sm:grid-cols-2">
          {doctor.checks.map((item) => (
            <article
              key={item.id}
              className="loom-card min-w-0 rounded-2xl p-4 transition-transform duration-150 hover:-translate-y-0.5 motion-reduce:transition-none motion-reduce:transform-none"
            >
              <div className="flex items-center justify-between gap-2">
                <div className="flex min-w-0 items-center gap-2.5">
                  <span className="loom-elevated inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-[var(--text-secondary)]">
                    <BrandIcon id={item.id} className="h-4 w-4" />
                  </span>
                  <h3 className="truncate text-sm font-medium text-[var(--text-primary)]">
                    {item.name}
                  </h3>
                </div>
                <span
                  className={`shrink-0 rounded-md px-2 py-0.5 font-mono text-xs ${
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
              <p className="mt-2 truncate font-mono text-xs text-[var(--text-secondary)]">
                {item.version ?? '-'} · {item.latency_ms} ms
              </p>
              {item.warnings.map((w) => (
                <p key={w} className="mt-1 break-words text-xs text-[var(--accent-amber)]">
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
              className="loom-elevated flex min-w-0 items-center justify-between gap-2 rounded-xl px-3 py-2.5 text-xs transition-transform duration-150 hover:-translate-y-0.5 motion-reduce:transition-none motion-reduce:transform-none"
            >
              <div className="flex min-w-0 items-center gap-2.5">
                <span className="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface)] text-[var(--text-secondary)]">
                  <BrandIcon id={p.id} className="h-3.5 w-3.5" />
                </span>
                <div className="min-w-0">
                  <p className="truncate font-medium text-[var(--text-primary)]">{p.name}</p>
                  <p className="truncate font-mono text-[var(--text-muted)]">
                    {p.tier} · {p.privacy}
                  </p>
                </div>
              </div>
              <span
                aria-hidden="true"
                className={`h-2 w-2 shrink-0 rounded-full ${
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
