import { useEffect, useRef } from 'react'
import { RefreshCw, WifiOff } from 'lucide-react'
import { useI18n, type LocaleCode } from '../i18n'
import type { MissionView } from '../types/api'

interface OfflineCanvasProps {
  lastSync: string | null
  cachedMissions: MissionView[]
  simulated: boolean
  onRetry: () => void
  onExitSimulated: () => void
}

const TELAR_PHILOSOPHY: Record<LocaleCode, string> = {
  es: 'Cada pausa sostiene la urdimbre hasta que el nodo vuelve a tejer.',
  en: 'Every pause holds the warp steady until the node weaves again.',
  'pt-BR': 'Cada pausa sustenta a urdidura até que o nó volte a tecer.',
}

interface WeftBeam {
  rowRatio: number
  speed: number
  phase: number
  token: '--accent-blue' | '--accent-green' | '--accent-amber'
}

const WEFT_BEAMS: readonly WeftBeam[] = [
  { rowRatio: 0.28, speed: 0.018, phase: 0, token: '--accent-blue' },
  { rowRatio: 0.5, speed: 0.013, phase: 2.1, token: '--accent-green' },
  { rowRatio: 0.72, speed: 0.021, phase: 4.2, token: '--accent-amber' },
]

function resolveToken(styles: CSSStyleDeclaration, token: string): string {
  return styles.getPropertyValue(token).trim() || 'currentColor'
}

export function OfflineCanvas({
  lastSync,
  cachedMissions,
  simulated,
  onRetry,
  onExitSimulated,
}: OfflineCanvasProps) {
  const { locale, t } = useI18n()
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
    let cancelled = false

    const draw = () => {
      if (!ctx || cancelled) return
      const w = canvas.width
      const h = canvas.height
      ctx.clearRect(0, 0, w, h)

      const styles = getComputedStyle(document.documentElement)
      const warpColor = resolveToken(styles, '--border-strong')
      const subtleColor = resolveToken(styles, '--border-subtle')
      const pointerColor = resolveToken(styles, '--accent-blue')

      const warpCount = 20
      const warpSpacing = w / (warpCount + 1)
      const px = pointerRef.current.active ? pointerRef.current.x * w : w * 0.5
      const py = pointerRef.current.active ? pointerRef.current.y * h : h * 0.5

      // Vertical taut warp threads (urdimbre) with subtle pointer tension deflection
      for (let i = 1; i <= warpCount; i++) {
        const baseX = i * warpSpacing
        const dx = baseX - px
        const distRatio = Math.max(0, 1 - Math.abs(dx) / (w * 0.24))
        const pull = pointerRef.current.active ? Math.sign(dx) * distRatio * 7 : 0

        ctx.beginPath()
        ctx.moveTo(baseX, 0)
        ctx.quadraticCurveTo(baseX + pull, py, baseX, h)
        ctx.strokeStyle = distRatio > 0.45 && pointerRef.current.active ? pointerColor : warpColor
        ctx.globalAlpha = distRatio > 0.45 && pointerRef.current.active ? 0.55 : 0.35
        ctx.lineWidth = 1
        ctx.stroke()
      }

      // Faint horizontal reference guide threads
      for (const beam of WEFT_BEAMS) {
        const y = beam.rowRatio * h
        ctx.beginPath()
        ctx.moveTo(0, y)
        ctx.lineTo(w, y)
        ctx.strokeStyle = subtleColor
        ctx.globalAlpha = 0.45
        ctx.lineWidth = 1
        ctx.stroke()
      }

      // Luminous horizontal weft shuttle beams (trama) weaving over/under warp threads
      for (let b = 0; b < WEFT_BEAMS.length; b++) {
        const beam = WEFT_BEAMS[b]
        if (!beam) continue
        const beamColor = resolveToken(styles, beam.token)
        const baseY = beam.rowRatio * h
        const shuttleNorm = 0.5 + 0.46 * Math.sin(tick * beam.speed + beam.phase)
        const shuttleX = shuttleNorm * w

        ctx.beginPath()
        for (let x = 0; x <= w; x += 6) {
          const warpStep = x / warpSpacing
          const overUnder = Math.sin(warpStep * Math.PI + b * Math.PI * 0.5) * 4.5
          const pointerDist = Math.hypot(x - px, baseY - py)
          const ripple = pointerRef.current.active
            ? Math.max(0, 1 - pointerDist / (w * 0.22)) * Math.sin(tick * 0.14 - pointerDist * 0.08) * 6
            : 0
          const y = baseY + overUnder + ripple
          if (x === 0) ctx.moveTo(x, y)
          else ctx.lineTo(x, y)
        }
        ctx.strokeStyle = beamColor
        ctx.globalAlpha = 0.72
        ctx.lineWidth = 1.75
        ctx.stroke()

        // Crossing intersection knots near shuttle head
        for (let i = 1; i <= warpCount; i++) {
          const wx = i * warpSpacing
          const proximity = Math.max(0, 1 - Math.abs(wx - shuttleX) / (w * 0.18))
          if (proximity <= 0.05) continue
          const overUnder = Math.sin(i * Math.PI + b * Math.PI * 0.5) * 4.5
          ctx.beginPath()
          ctx.arc(wx, baseY + overUnder, 1.5 + proximity * 2.5, 0, Math.PI * 2)
          ctx.fillStyle = beamColor
          ctx.globalAlpha = 0.3 + proximity * 0.7
          ctx.fill()
        }

        // Luminous shuttle head
        const shuttleOverUnder = Math.sin((shuttleX / warpSpacing) * Math.PI + b * Math.PI * 0.5) * 4.5
        ctx.beginPath()
        ctx.arc(shuttleX, baseY + shuttleOverUnder, 4.5, 0, Math.PI * 2)
        ctx.fillStyle = beamColor
        ctx.globalAlpha = 0.95
        ctx.fill()
      }

      ctx.globalAlpha = 1

      if (!reduced && !document.hidden) {
        tick += 1
        rafId = window.requestAnimationFrame(draw)
      }
    }

    const onVisibility = () => {
      if (!document.hidden && !reduced && !rafId && !cancelled) {
        rafId = window.requestAnimationFrame(draw)
      }
    }

    document.addEventListener('visibilitychange', onVisibility)
    draw()

    return () => {
      cancelled = true
      document.removeEventListener('visibilitychange', onVisibility)
      if (rafId) window.cancelAnimationFrame(rafId)
    }
  }, [])

  return (
    <div className="mx-auto flex min-h-[75vh] max-w-2xl flex-col justify-center gap-5 px-4 py-8">
      <div className="loom-card rounded-3xl p-5 sm:p-6">
        <div className="flex items-center gap-3">
          <span className="inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl bg-[var(--accent-amber-soft)] text-[var(--accent-amber)]">
            <WifiOff className="h-5 w-5" aria-hidden="true" />
          </span>
          <div className="min-w-0">
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

        <p className="mt-3 font-mono text-xs text-[var(--text-muted)]">
          {TELAR_PHILOSOPHY[locale]}
        </p>
        <p className="mt-2 text-sm text-[var(--text-secondary)]">{t('offline.hint')}</p>
        <p className="mt-1 font-mono text-xs text-[var(--text-muted)]">
          {t('offline.lastSync')}: {lastSync ?? t('offline.never')}
        </p>

        {cachedMissions.length > 0 && (
          <div className="mt-4 space-y-2 border-t border-[var(--border-subtle)] pt-4">
            {cachedMissions.slice(0, 3).map((m) => (
              <div
                key={m.id}
                className="loom-elevated flex items-center justify-between gap-2 rounded-xl px-3 py-2 text-xs"
              >
                <span className="truncate text-[var(--text-primary)]">{m.title}</span>
                <span className="shrink-0 font-mono text-[var(--text-muted)]">
                  {m.provider_id}
                </span>
              </div>
            ))}
          </div>
        )}

        <div className="mt-5 flex flex-wrap gap-3">
          <button
            type="button"
            onClick={simulated ? onExitSimulated : onRetry}
            className="inline-flex min-h-11 items-center gap-2 rounded-xl bg-[var(--accent-blue)] px-4 text-sm font-medium text-white transition-opacity hover:opacity-90 active:scale-95 focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
          >
            <RefreshCw className="h-4 w-4 shrink-0" aria-hidden="true" />
            {simulated ? t('action.exitOffline') : t('action.retry')}
          </button>
        </div>
      </div>
    </div>
  )
}
