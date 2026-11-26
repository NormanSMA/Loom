interface DotHeatmapProps {
  points: { day: string; count: number }[]
  title: string
}

export function DotHeatmap({ points, title }: DotHeatmapProps) {
  const tone = (count: number) => {
    if (count === 0) return 'bg-[var(--dot-inactive)]'
    if (count <= 3) return 'bg-[var(--accent-green-soft)] border border-[var(--accent-green)]'
    if (count <= 6) return 'bg-[var(--accent-blue)] opacity-75'
    return 'bg-[var(--accent-green)]'
  }

  return (
    <section className="loom-card rounded-2xl p-4">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-medium text-[var(--text-primary)]">{title}</h3>
        <span className="font-mono text-xs text-[var(--text-muted)]">14x7</span>
      </div>
      <div className="grid grid-cols-14 gap-1.5">
        {points.slice(-98).map((pt) => (
          <div
            key={pt.day}
            title={`${pt.day}: ${pt.count}`}
            aria-label={`${pt.day}: ${pt.count}`}
            className={`aspect-square rounded-[3px] transition-transform duration-150 hover:scale-110 ${tone(pt.count)}`}
          />
        ))}
      </div>
    </section>
  )
}
