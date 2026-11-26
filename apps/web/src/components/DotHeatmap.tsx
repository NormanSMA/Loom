import { useI18n } from '../i18n'

interface DotHeatmapProps {
  points: { day: string; count: number }[]
  title: string
}

export function DotHeatmap({ points, title }: DotHeatmapProps) {
  const { t } = useI18n()

  const cells: { day: string; count: number }[] = []
  const tail = points.slice(-98)
  const padCount = Math.max(0, 98 - tail.length)
  for (let i = 0; i < padCount; i++) {
    cells.push({ day: `empty-${i}`, count: 0 })
  }
  cells.push(...tail)

  const tone = (count: number) => {
    if (count === 0) return 'bg-[var(--dot-inactive)]'
    if (count <= 3) return 'bg-[var(--accent-green-soft)] border border-[var(--accent-green)]'
    if (count <= 6) return 'bg-[var(--accent-blue)] opacity-75'
    return 'bg-[var(--accent-green)]'
  }

  return (
    <section className="loom-card rounded-2xl p-4">
      <div className="mb-3 flex items-center justify-between gap-2">
        <h3 className="text-sm font-medium text-[var(--text-primary)]">{title}</h3>
        <div className="flex items-center gap-1.5 font-mono text-[10px] text-[var(--text-muted)]">
          <span>{t('telemetry.less')}</span>
          <span className="h-2.5 w-2.5 rounded-[2px] bg-[var(--dot-inactive)]" />
          <span className="h-2.5 w-2.5 rounded-[2px] border border-[var(--accent-green)] bg-[var(--accent-green-soft)]" />
          <span className="h-2.5 w-2.5 rounded-[2px] bg-[var(--accent-blue)] opacity-75" />
          <span className="h-2.5 w-2.5 rounded-[2px] bg-[var(--accent-green)]" />
          <span>{t('telemetry.more')}</span>
        </div>
      </div>
      <div className="grid grid-cols-14 gap-1 sm:gap-1.5">
        {cells.map((pt) => (
          <div
            key={pt.day}
            title={pt.day.startsWith('empty-') ? undefined : `${pt.day}: ${pt.count}`}
            aria-label={pt.day.startsWith('empty-') ? undefined : `${pt.day}: ${pt.count}`}
            className={`aspect-square rounded-[3px] transition-transform duration-150 hover:scale-110 ${tone(pt.count)}`}
          />
        ))}
      </div>
    </section>
  )
}
