import { useEffect, useRef } from 'react'
import { RefreshCw, WifiOff } from 'lucide-react'
import { useI18n } from '../i18n'
import type { MissionView } from '../types/api'

interface OfflineCanvasProps {
  lastSync: string | null
  cachedMissions: MissionView[]
  simulated: boolean
  onRetry: () => void
  onExitSimulated: () => void
}

export function OfflineCanvas({
  lastSync,
  cachedMissions,
  simulated,
  onRetry,
  onExitSimulated,
}: OfflineCanvasProps) {
  const { t } = useI18n()
  const canvasRef = useRef<HTMLCanvasElement | null>(null)
  const pointerRef = useRef<{ x: number; y: number; active: boolean }>({
    x: 0.5,
    y: 0.5,
    active: false,
  })

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    let ctx: CanvasRenderingContext2D | null = null
    try {
      ctx = canvas.getContext('2d')
    } catch {
      ctx = null
    }
    if (!ctx) return

    const reduced =
      typeof window.matchMedia === 'function' &&
      window.matchMedia('(prefers-reduced-motion: reduce)').matches

    let rafId = 0
    let tick = 0

    const draw = () => {
      if (!ctx) return
      const w = canvas.width
      const h = canvas.height
      ctx.clearRect(0, 0, w, h)

      const styles = getComputedStyle(document.documentElement)
      const inactiveColor = styles.getPropertyValue('--border-strong').trim() || '#3a3a3a'
      const activeColor = styles.getPropertyValue('--accent-blue').trim() || '#3b82f6'

      const cols = 16
      const rows = 8
      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
          const nx = (c + 0.5) / cols
          const ny = (r + 0.5) / rows
          const px = pointerRef.current.active
            ? pointerRef.current.x
            : 0.5 + Math.cos(tick * 0.03) * 0.25
          const py = pointerRef.current.active
            ? pointerRef.current.y
            : 0.5 + Math.sin(tick * 0.04) * 0.2
          const dist = Math.hypot(nx - px, ny - py)
          const boost = Math.max(0, 1 - dist * 3.2)

          ctx.beginPath()
          ctx.arc(nx * w, ny * h, 2 + boost * 4.5, 0, Math.PI * 2)
          ctx.fillStyle = boost > 0.35 ? activeColor : inactiveColor
          ctx.globalAlpha = 0.35 + boost * 0.65
          ctx.fill()
        }
      }
      ctx.globalAlpha = 1

      if (!reduced && !document.hidden) {
        tick += 1
        rafId = window.requestAnimationFrame(draw)
      }
    }

    draw()
    return () => {
      if (rafId) window.cancelAnimationFrame(rafId)
    }
  }, [])

  return (
    <div className="mx-auto flex min-h-[75vh] max-w-2xl flex-col justify-center gap-5 px-4 py-8">
      <div className="loom-card rounded-3xl p-6">
        <div className="flex items-center gap-3">
          <span className="inline-flex h-11 w-11 items-center justify-center rounded-2xl bg-[var(--accent-amber-soft)] text-[var(--accent-amber)]">
            <WifiOff className="h-5 w-5" />
          </span>
          <div>
            <p className="font-mono text-xs uppercase tracking-widest text-[var(--accent-amber)]">
              {t('status.offline')}
            </p>
            <h1 className="text-lg font-semibold text-[var(--text-primary)]">
              {t('offline.title')}
            </h1>
          </div>
        </div>

        <div className="loom-elevated mt-5 overflow-hidden rounded-2xl p-2">
          <canvas
            ref={canvasRef}
            width={560}
            height={210}
            onPointerMove={(e) => {
              const rect = e.currentTarget.getBoundingClientRect()
              pointerRef.current = {
                x: (e.clientX - rect.left) / Math.max(1, rect.width),
                y: (e.clientY - rect.top) / Math.max(1, rect.height),
                active: true,
              }
            }}
            onPointerLeave={() => {
              pointerRef.current.active = false
            }}
            className="h-44 w-full touch-none rounded-xl"
          />
        </div>

        <p className="mt-4 text-sm text-[var(--text-secondary)]">{t('offline.hint')}</p>
        <p className="mt-1 font-mono text-xs text-[var(--text-muted)]">
          {t('offline.lastSync')}: {lastSync ?? t('offline.never')}
        </p>

        {cachedMissions.length > 0 && (
          <div className="mt-4 space-y-2 border-t border-[var(--border-subtle)] pt-4">
            {cachedMissions.slice(0, 3).map((m) => (
              <div
                key={m.id}
                className="loom-elevated flex items-center justify-between rounded-xl px-3 py-2 text-xs"
              >
                <span className="truncate text-[var(--text-primary)]">{m.title}</span>
                <span className="font-mono text-[var(--text-muted)]">{m.provider_id}</span>
              </div>
            ))}
          </div>
        )}

        <div className="mt-5 flex flex-wrap gap-3">
          <button
            type="button"
            onClick={simulated ? onExitSimulated : onRetry}
            className="inline-flex min-h-11 items-center gap-2 rounded-xl bg-[var(--accent-blue)] px-4 text-sm font-medium text-white focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
          >
            <RefreshCw className="h-4 w-4" />
            {simulated ? t('action.exitOffline') : t('action.retry')}
          </button>
        </div>
      </div>
    </div>
  )
}
