import { useMemo, useState } from 'react'

export type MatrixTone = 'green' | 'amber' | 'blue'

export interface DotMatrixRingStat {
  label: string
  value: string
  tone?: MatrixTone
}

export interface DotMatrixRingProps {
  ratio: number
  valueLabel: string
  centerValue: string
  caption?: string
  title?: string
  tone?: MatrixTone
  stats?: DotMatrixRingStat[]
}

export interface DotMatrixPoint {
  day: string
  count: number
}

export interface DotMatrixEqualizerProps {
  points: DotMatrixPoint[]
  title: string
  subtitle?: string
  unitLabel?: string
  emptyLabel?: string
}

export interface DotMatrixSparklineLabels {
  avgPerDay: string
  peakDay: string
  activeDays: string
  totalCalls: string
}

export interface DotMatrixSparklineProps {
  points: DotMatrixPoint[]
  title: string
  subtitle?: string
  totalOverride?: number
  labels: DotMatrixSparklineLabels
}

const TONE_CSS_VAR: Record<MatrixTone, string> = {
  green: 'var(--accent-green)',
  amber: 'var(--accent-amber)',
  blue: 'var(--accent-blue)',
}

const TONE_BG_CLASS: Record<MatrixTone, string> = {
  green: 'bg-[var(--accent-green)]',
  amber: 'bg-[var(--accent-amber)]',
  blue: 'bg-[var(--accent-blue)]',
}

const OUTER_RING_COUNT = 24
const INNER_RING_COUNT = 12
const EQUALIZER_COLS = 14
const EQUALIZER_ROWS = 8

function clampRatio(value: number): number {
  if (!Number.isFinite(value)) return 0
  return Math.max(0, Math.min(1, value))
}

export function DotMatrixRing({
  ratio,
  valueLabel,
  centerValue,
  caption,
  title,
  tone = 'green',
  stats = [],
}: DotMatrixRingProps) {
  const safeRatio = clampRatio(ratio)
  const outerActive = Math.round(safeRatio * OUTER_RING_COUNT)
  const innerActive = Math.round(safeRatio * INNER_RING_COUNT)
  const activeColor = TONE_CSS_VAR[tone]

  const outerDots = useMemo(
    () =>
      Array.from({ length: OUTER_RING_COUNT }, (_, idx) => {
        const angle = (idx / OUTER_RING_COUNT) * Math.PI * 2 - Math.PI / 2
        return {
          idx,
          cx: Number((56 + Math.cos(angle) * 44).toFixed(2)),
          cy: Number((56 + Math.sin(angle) * 44).toFixed(2)),
        }
      }),
    [],
  )

  const innerDots = useMemo(
    () =>
      Array.from({ length: INNER_RING_COUNT }, (_, idx) => {
        const angle = (idx / INNER_RING_COUNT) * Math.PI * 2 - Math.PI / 2
        return {
          idx,
          cx: Number((56 + Math.cos(angle) * 32).toFixed(2)),
          cy: Number((56 + Math.sin(angle) * 32).toFixed(2)),
        }
      }),
    [],
  )

  return (
    <section
      tabIndex={0}
      aria-label={title ?? valueLabel}
      className="loom-card group rounded-2xl p-4 transition-colors duration-200 hover:border-[var(--accent-blue)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] motion-reduce:transition-none"
    >
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center">
        <div className="relative mx-auto flex h-28 w-28 shrink-0 items-center justify-center sm:mx-0">
          <div
            aria-hidden="true"
            className="pointer-events-none absolute inset-2 rounded-full bg-[var(--accent-blue-soft)] opacity-0 blur-md transition-opacity duration-200 group-hover:opacity-80 group-focus-visible:opacity-80 motion-reduce:transition-none"
          />
          <svg
            viewBox="0 0 112 112"
            className="relative h-28 w-28"
            role="img"
            aria-label={`${valueLabel}: ${centerValue}`}
          >
            {outerDots.map((dot) => {
              const lit = dot.idx < outerActive
              return (
                <circle
                  key={`o-${dot.idx}`}
                  cx={dot.cx}
                  cy={dot.cy}
                  r={lit ? 3.1 : 2.5}
                  fill={lit ? activeColor : 'var(--dot-inactive)'}
                />
              )
            })}
            {innerDots.map((dot) => {
              const lit = dot.idx < innerActive
              return (
                <circle
                  key={`i-${dot.idx}`}
                  cx={dot.cx}
                  cy={dot.cy}
                  r={2}
                  fill={lit ? activeColor : 'var(--dot-inactive)'}
                  opacity={lit ? 0.65 : 0.7}
                />
              )
            })}
          </svg>
          <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center text-center">
            <span className="font-mono text-lg font-semibold leading-none text-[var(--text-primary)] tabular-nums">
              {centerValue}
            </span>
            <span className="mt-1 font-mono text-xs uppercase tracking-wider text-[var(--text-muted)]">
              {valueLabel}
            </span>
          </div>
        </div>

        <div className="min-w-0 flex-1 space-y-2.5">
          {(title || caption) && (
            <div className="min-w-0">
              {title && (
                <h2 className="truncate text-sm font-medium text-[var(--text-primary)]">
                  {title}
                </h2>
              )}
              {caption && (
                <p className="truncate font-mono text-xs text-[var(--text-muted)]">{caption}</p>
              )}
            </div>
          )}

          {stats.length > 0 && (
            <div className="grid grid-cols-2 gap-2">
              {stats.map((stat) => {
                const dotClass = TONE_BG_CLASS[stat.tone ?? 'blue']
                return (
                  <div
                    key={stat.label}
                    className="loom-elevated min-w-0 rounded-xl px-2.5 py-2"
                  >
                    <div className="flex items-center gap-1.5">
                      <span className={`h-1.5 w-1.5 shrink-0 rounded-full ${dotClass}`} />
                      <span className="truncate text-xs text-[var(--text-muted)]">
                        {stat.label}
                      </span>
                    </div>
                    <p className="mt-0.5 truncate font-mono text-xs font-semibold text-[var(--text-primary)] tabular-nums">
                      {stat.value}
                    </p>
                  </div>
                )
              })}
            </div>
          )}
        </div>
      </div>
    </section>
  )
}

export function DotMatrixEqualizer({
  points,
  title,
  subtitle,
  unitLabel = '',
  emptyLabel = '--',
}: DotMatrixEqualizerProps) {
  const columns = useMemo(() => {
    const tail = points.slice(-EQUALIZER_COLS)
    const padCount = Math.max(0, EQUALIZER_COLS - tail.length)
    const padded: (DotMatrixPoint & { placeholder: boolean })[] = []
    for (let i = 0; i < padCount; i += 1) {
      padded.push({ day: `pad-${i}`, count: 0, placeholder: true })
    }
    for (const pt of tail) {
      padded.push({ day: pt.day, count: pt.count, placeholder: false })
    }
    return padded
  }, [points])

  const defaultIndex = useMemo(() => {
    for (let i = columns.length - 1; i >= 0; i -= 1) {
      const col = columns[i]
      if (col && !col.placeholder && col.count > 0) return i
    }
    return columns.length - 1
  }, [columns])

  const [selectedIndex, setSelectedIndex] = useState<number | null>(null)
  const activeIndex = selectedIndex ?? defaultIndex
  const activePoint = columns[activeIndex] ??
    columns[columns.length - 1] ?? { day: '', count: 0, placeholder: true }
  const maxCount = Math.max(1, ...columns.map((c) => c.count))

  const rows = useMemo(
    () => Array.from({ length: EQUALIZER_ROWS }, (_, idx) => EQUALIZER_ROWS - 1 - idx),
    [],
  )

  const firstRealDay = columns.find((c) => !c.placeholder)?.day ?? ''
  const lastRealDay = [...columns].reverse().find((c) => !c.placeholder)?.day ?? ''

  return (
    <section className="loom-card min-w-0 rounded-2xl p-4">
      <div className="mb-3 flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0">
          <h3 className="truncate text-sm font-medium text-[var(--text-primary)]">{title}</h3>
          {subtitle && (
            <p className="truncate text-xs text-[var(--text-muted)]">{subtitle}</p>
          )}
        </div>
        <div
          aria-live="polite"
          className="loom-elevated inline-flex min-h-7 shrink-0 items-center gap-2 rounded-lg px-2.5 py-1 font-mono text-xs tabular-nums"
        >
          <span className="text-[var(--text-muted)]">
            {activePoint.placeholder ? emptyLabel : activePoint.day}
          </span>
          <span className="font-semibold text-[var(--accent-blue)]">
            {activePoint.placeholder ? emptyLabel : `${activePoint.count}${unitLabel ? ` ${unitLabel}` : ''}`}
          </span>
        </div>
      </div>

      <div className="grid grid-cols-14 gap-1 sm:gap-1.5" role="group" aria-label={title}>
        {columns.map((col, colIdx) => {
          const isFocused = colIdx === activeIndex
          const litDots =
            col.placeholder || col.count <= 0
              ? 0
              : Math.max(1, Math.round((col.count / maxCount) * EQUALIZER_ROWS))

          return (
            <button
              key={col.day}
              type="button"
              onClick={() => setSelectedIndex(colIdx)}
              onMouseEnter={() => setSelectedIndex(colIdx)}
              onFocus={() => setSelectedIndex(colIdx)}
              aria-pressed={isFocused}
              aria-label={col.placeholder ? emptyLabel : `${col.day}: ${col.count}`}
              className={`flex min-h-11 min-w-0 flex-col items-center justify-end gap-1 rounded-lg py-1.5 transition-colors duration-150 focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] motion-reduce:transition-none ${
                isFocused
                  ? 'bg-[var(--accent-blue-soft)]'
                  : 'hover:bg-[var(--bg-elevated)]'
              }`}
            >
              {rows.map((rowIdx) => {
                const isLit = rowIdx < litDots
                const isPeak = isLit && rowIdx >= EQUALIZER_ROWS - 2
                let dotColor = 'bg-[var(--dot-inactive)]'
                if (isLit) {
                  if (isFocused) {
                    dotColor = isPeak ? 'bg-[var(--accent-amber)]' : 'bg-[var(--accent-blue)]'
                  } else {
                    dotColor = isPeak ? 'bg-[var(--accent-blue)]' : 'bg-[var(--accent-green)]'
                  }
                }
                return (
                  <span
                    key={rowIdx}
                    className={`h-1.5 w-1.5 rounded-full transition-colors duration-150 sm:h-2 sm:w-2 motion-reduce:transition-none ${dotColor}`}
                  />
                )
              })}
            </button>
          )
        })}
      </div>

      <div className="mt-2 flex items-center justify-between font-mono text-xs text-[var(--text-muted)]">
        <span className="truncate">{firstRealDay || emptyLabel}</span>
        <span className="truncate">{lastRealDay || emptyLabel}</span>
      </div>
    </section>
  )
}

export function DotMatrixSparkline({
  points,
  title,
  subtitle,
  totalOverride,
  labels,
}: DotMatrixSparklineProps) {
  const recent = useMemo(() => points.slice(-28), [points])

  const stats = useMemo(() => {
    if (recent.length === 0) {
      return {
        avgPerDay: '0.0',
        peakCount: 0,
        peakDay: '--',
        activeDays: 0,
        total: totalOverride ?? 0,
        peakIndex: -1,
        maxCount: 1,
      }
    }

    let sum = 0
    let activeDays = 0
    let peakCount = -1
    let peakIndex = 0
    for (let i = 0; i < recent.length; i += 1) {
      const item = recent[i]
      if (!item) continue
      const val = item.count
      sum += val
      if (val > 0) activeDays += 1
      if (val >= peakCount) {
        peakCount = val
        peakIndex = i
      }
    }

    return {
      avgPerDay: (sum / recent.length).toFixed(1),
      peakCount: Math.max(0, peakCount),
      peakDay: recent[peakIndex]?.day ?? '--',
      activeDays,
      total: totalOverride ?? sum,
      peakIndex,
      maxCount: Math.max(1, peakCount),
    }
  }, [recent, totalOverride])

  const coords = useMemo(() => {
    if (recent.length === 0) return []
    const width = 280
    const padX = 10
    const usableW = width - padX * 2
    const stepX = recent.length > 1 ? usableW / (recent.length - 1) : 0
    const topY = 12
    const bottomY = 74
    const usableH = bottomY - topY

    return recent.map((pt, idx) => {
      const x = recent.length === 1 ? width / 2 : padX + idx * stepX
      const ratio = pt.count / stats.maxCount
      const y = bottomY - ratio * usableH
      return {
        day: pt.day,
        count: pt.count,
        x: Number(x.toFixed(1)),
        y: Number(y.toFixed(1)),
      }
    })
  }, [recent, stats.maxCount])

  const polylinePoints = useMemo(
    () => coords.map((pt) => `${pt.x},${pt.y}`).join(' '),
    [coords],
  )

  const gridDots = useMemo(() => {
    const dots: { x: number; y: number }[] = []
    for (let row = 0; row < 5; row += 1) {
      const y = 14 + row * 15
      for (let col = 0; col < 19; col += 1) {
        const x = 10 + col * 14.4
        dots.push({ x: Number(x.toFixed(1)), y })
      }
    }
    return dots
  }, [])

  return (
    <section className="loom-card min-w-0 rounded-2xl p-4">
      <div className="mb-3 flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0">
          <h3 className="truncate text-sm font-medium text-[var(--text-primary)]">{title}</h3>
          {subtitle && (
            <p className="truncate text-xs text-[var(--text-muted)]">{subtitle}</p>
          )}
        </div>
        {stats.peakCount > 0 && (
          <span className="rounded-full bg-[var(--accent-amber-soft)] px-2.5 py-0.5 font-mono text-xs text-[var(--accent-amber)] tabular-nums">
            {labels.peakDay}: {stats.peakCount}
          </span>
        )}
      </div>

      <div className="loom-elevated overflow-hidden rounded-xl px-2 py-2">
        <svg
          viewBox="0 0 280 86"
          className="h-24 w-full"
          role="img"
          aria-label={title}
        >
          {gridDots.map((dot, idx) => (
            <circle
              key={idx}
              cx={dot.x}
              cy={dot.y}
              r={1}
              fill="var(--dot-inactive)"
              opacity={0.75}
            />
          ))}

          {coords.length > 1 && (
            <polyline
              fill="none"
              stroke="var(--accent-blue)"
              strokeWidth={1.6}
              strokeDasharray="2 3.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              points={polylinePoints}
            />
          )}

          {coords.map((pt, idx) => {
            const isPeak = idx === stats.peakIndex && pt.count > 0
            return (
              <g key={pt.day}>
                {pt.count > 0 && (
                  <line
                    x1={pt.x}
                    y1={pt.y}
                    x2={pt.x}
                    y2={74}
                    stroke="var(--accent-blue)"
                    strokeWidth={1}
                    strokeDasharray="1.5 3"
                    opacity={0.4}
                  />
                )}
                {isPeak && (
                  <circle
                    cx={pt.x}
                    cy={pt.y}
                    r={5.5}
                    fill="none"
                    stroke="var(--accent-amber)"
                    strokeWidth={1.2}
                    opacity={0.75}
                  />
                )}
                <circle
                  cx={pt.x}
                  cy={pt.y}
                  r={isPeak ? 3 : pt.count > 0 ? 2.2 : 1.5}
                  fill={
                    isPeak
                      ? 'var(--accent-amber)'
                      : pt.count > 0
                        ? 'var(--accent-green)'
                        : 'var(--dot-inactive)'
                  }
                />
              </g>
            )
          })}
        </svg>
      </div>

      <div className="mt-3 grid grid-cols-2 gap-2 sm:grid-cols-4">
        <div className="loom-elevated min-w-0 rounded-xl px-2.5 py-2">
          <p className="truncate text-xs uppercase tracking-wider text-[var(--text-muted)]">
            {labels.avgPerDay}
          </p>
          <p className="mt-0.5 truncate font-mono text-xs font-semibold text-[var(--text-primary)] tabular-nums">
            {stats.avgPerDay}
          </p>
        </div>
        <div className="loom-elevated min-w-0 rounded-xl px-2.5 py-2">
          <p className="truncate text-xs uppercase tracking-wider text-[var(--text-muted)]">
            {labels.peakDay}
          </p>
          <p className="mt-0.5 truncate font-mono text-xs font-semibold text-[var(--accent-amber)] tabular-nums">
            {stats.peakCount}
          </p>
        </div>
        <div className="loom-elevated min-w-0 rounded-xl px-2.5 py-2">
          <p className="truncate text-xs uppercase tracking-wider text-[var(--text-muted)]">
            {labels.activeDays}
          </p>
          <p className="mt-0.5 truncate font-mono text-xs font-semibold text-[var(--accent-green)] tabular-nums">
            {stats.activeDays}/{recent.length}
          </p>
        </div>
        <div className="loom-elevated min-w-0 rounded-xl px-2.5 py-2">
          <p className="truncate text-xs uppercase tracking-wider text-[var(--text-muted)]">
            {labels.totalCalls}
          </p>
          <p className="mt-0.5 truncate font-mono text-xs font-semibold text-[var(--accent-blue)] tabular-nums">
            {stats.total.toLocaleString()}
          </p>
        </div>
      </div>
    </section>
  )
}
