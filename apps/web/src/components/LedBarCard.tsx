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
    <article className="loom-card rounded-2xl p-4 transition-transform duration-150 hover:-translate-y-0.5">
      <div className="flex items-baseline justify-between gap-2">
        <span className="text-xs uppercase tracking-wider text-[var(--text-muted)]">
          {label}
        </span>
        {hint && <span className="font-mono text-xs text-[var(--text-secondary)]">{hint}</span>}
      </div>
      <p className="mt-1 font-mono text-2xl font-semibold text-[var(--text-primary)]">{value}</p>
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
