interface LedBarCardProps {
  label: string
  value: string
  fillRatio: number
  tone: 'blue' | 'green' | 'amber'
  hint?: string
}

const SEGMENTS = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

export function LedBarCard({ label, value, fillRatio, tone, hint }: LedBarCardProps) {
  const activeCount = Math.max(0, Math.min(10, Math.round(fillRatio * 10)))
  const activeClass =
    tone === 'green'
      ? 'bg-[var(--accent-green)]'
      : tone === 'amber'
        ? 'bg-[var(--accent-amber)]'
        : 'bg-[var(--accent-blue)]'

  return (
    <article className="loom-card min-w-0 rounded-2xl p-3.5 transition-transform duration-150 hover:-translate-y-0.5 sm:p-4">
      <div className="flex min-w-0 items-baseline justify-between gap-1.5">
        <span className="truncate text-[11px] uppercase tracking-wider text-[var(--text-muted)] sm:text-xs">
          {label}
        </span>
        {hint && (
          <span className="hidden truncate font-mono text-[10px] text-[var(--text-secondary)] sm:inline">
            {hint}
          </span>
        )}
      </div>
      <p className="mt-1 truncate font-mono text-xl font-semibold text-[var(--text-primary)] sm:text-2xl">
        {value}
      </p>
      <div className="mt-3 grid grid-cols-10 gap-1" aria-hidden="true">
        {SEGMENTS.map((idx) => (
          <span
            key={idx}
            className={`h-2 rounded-xs ${idx < activeCount ? activeClass : 'bg-[var(--dot-inactive)]'}`}
          />
        ))}
      </div>
    </article>
  )
}
