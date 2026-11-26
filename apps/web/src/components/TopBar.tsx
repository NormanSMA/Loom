import { useId, useState } from 'react'
import { OctagonAlert, Settings } from 'lucide-react'
import { useI18n, type LocaleCode } from '../i18n'

export interface TopBarProps {
  demo: boolean
  remoteEnabled: boolean
  onOpenSettings: () => void
  onEmergencyStop: () => Promise<void>
}

interface TopBarCopy {
  liveLocal: string
  liveTunnel: string
  demoTag: string
  localTooltip: string
  tunnelTooltip: string
  demoTooltip: string
  settingsLabel: string
  killHint: string
}

const TOPBAR_COPY: Record<LocaleCode, TopBarCopy> = {
  es: {
    liveLocal: 'LIVE · LOCAL',
    liveTunnel: 'LIVE · TÚNEL',
    demoTag: 'DEMO',
    localTooltip: 'Red local 127.0.0.1:4320',
    tunnelTooltip: 'Túnel remoto cifrado',
    demoTooltip: 'Modo demo con datos locales simulados',
    settingsLabel: 'Ajustes y perfil',
    killHint: 'Detiene de inmediato los agentes y worktrees activos',
  },
  en: {
    liveLocal: 'LIVE · LOCAL',
    liveTunnel: 'LIVE · TUNNEL',
    demoTag: 'DEMO',
    localTooltip: 'Local network 127.0.0.1:4320',
    tunnelTooltip: 'Encrypted remote tunnel',
    demoTooltip: 'Demo mode with simulated local data',
    settingsLabel: 'Settings & profile',
    killHint: 'Immediately stops all active agents and worktrees',
  },
  'pt-BR': {
    liveLocal: 'LIVE · LOCAL',
    liveTunnel: 'LIVE · TÚNEL',
    demoTag: 'DEMO',
    localTooltip: 'Rede local 127.0.0.1:4320',
    tunnelTooltip: 'Túnel remoto criptografado',
    demoTooltip: 'Modo demo com dados locais simulados',
    settingsLabel: 'Ajustes e perfil',
    killHint: 'Interrompe imediatamente todos os agentes e worktrees ativos',
  },
}

export function TopBar({
  demo,
  remoteEnabled,
  onOpenSettings,
  onEmergencyStop,
}: TopBarProps) {
  const { locale, t } = useI18n()
  const [stopping, setStopping] = useState(false)
  const signalTooltipId = useId()
  const settingsTooltipId = useId()
  const killTooltipId = useId()

  const copy = TOPBAR_COPY[locale]

  const badgeLabel = demo
    ? copy.demoTag
    : remoteEnabled
      ? copy.liveTunnel
      : copy.liveLocal

  const badgeTooltip = demo
    ? copy.demoTooltip
    : remoteEnabled
      ? copy.tunnelTooltip
      : copy.localTooltip

  const accentBarClass = demo
    ? 'bg-[var(--accent-amber)]'
    : remoteEnabled
      ? 'bg-[var(--accent-blue)]'
      : 'bg-[var(--accent-green)]'

  const handleStop = async () => {
    if (stopping) return
    setStopping(true)
    try {
      await onEmergencyStop()
    } finally {
      setStopping(false)
    }
  }

  return (
    <header className="sticky top-0 z-20 flex items-center justify-between gap-2 border-b border-[var(--border-subtle)] bg-[var(--bg-canvas)]/90 px-3 py-2 backdrop-blur-md sm:px-4">
      <div className="group relative flex min-w-0 items-center">
        <span
          tabIndex={0}
          title={badgeTooltip}
          aria-describedby={signalTooltipId}
          className="loom-card inline-flex min-h-9 cursor-default items-center gap-2 rounded-full px-3 font-mono text-xs text-[var(--text-primary)] transition-colors hover:border-[var(--border-strong)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
        >
          <span className="inline-flex h-3 items-end gap-0.5" aria-hidden="true">
            <span className={`signal-bar signal-bar-1 h-1.5 w-0.5 rounded-full ${accentBarClass}`} />
            <span className={`signal-bar signal-bar-2 h-2.5 w-0.5 rounded-full ${accentBarClass}`} />
            <span className={`signal-bar signal-bar-3 h-3 w-0.5 rounded-full ${accentBarClass}`} />
          </span>
          <span className="truncate font-medium tracking-wide">{badgeLabel}</span>
        </span>

        <div
          id={signalTooltipId}
          role="tooltip"
          className="glass-folder pointer-events-none invisible absolute top-full left-0 z-30 mt-2 w-max max-w-60 rounded-xl px-3 py-2 text-xs text-[var(--text-primary)] opacity-0 shadow-lg transition-opacity duration-150 group-hover:visible group-hover:opacity-100 group-focus-within:visible group-focus-within:opacity-100"
        >
          <p className="font-mono text-xs font-medium">{badgeLabel}</p>
          <p className="mt-0.5 text-xs text-[var(--text-secondary)]">{badgeTooltip}</p>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <div className="group relative">
          <button
            type="button"
            onClick={onOpenSettings}
            title={copy.settingsLabel}
            aria-label={copy.settingsLabel}
            aria-describedby={settingsTooltipId}
            className="loom-card inline-flex min-h-11 min-w-11 items-center justify-center rounded-xl text-[var(--text-primary)] transition-colors hover:border-[var(--border-strong)] hover:bg-[var(--bg-elevated)] active:scale-95 focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
          >
            <Settings className="h-4 w-4" aria-hidden="true" />
          </button>

          <div
            id={settingsTooltipId}
            role="tooltip"
            className="glass-folder pointer-events-none invisible absolute top-full right-0 z-30 mt-2 w-max max-w-56 rounded-xl px-3 py-2 text-xs text-[var(--text-primary)] opacity-0 shadow-lg transition-opacity duration-150 group-hover:visible group-hover:opacity-100 group-focus-within:visible group-focus-within:opacity-100"
          >
            {copy.settingsLabel}
          </div>
        </div>

        <div className="group relative">
          <button
            type="button"
            disabled={stopping}
            onClick={() => void handleStop()}
            title={`${t('action.killSwitch')} — ${copy.killHint}`}
            aria-label={t('action.killSwitch')}
            aria-describedby={killTooltipId}
            className="inline-flex min-h-11 min-w-11 items-center justify-center rounded-xl border border-[var(--accent-red)] bg-[var(--accent-red-soft)] text-[var(--accent-red)] transition-colors hover:bg-[var(--accent-red)] hover:text-white active:scale-95 disabled:opacity-50 focus-visible:outline-2 focus-visible:outline-[var(--accent-red)]"
          >
            <OctagonAlert className="h-4 w-4 shrink-0" aria-hidden="true" />
          </button>

          <div
            id={killTooltipId}
            role="tooltip"
            className="glass-folder pointer-events-none invisible absolute top-full right-0 z-30 mt-2 w-56 rounded-xl px-3 py-2 text-left text-xs text-[var(--text-primary)] opacity-0 shadow-lg transition-opacity duration-150 group-hover:visible group-hover:opacity-100 group-focus-within:visible group-focus-within:opacity-100"
          >
            <p className="font-semibold text-[var(--accent-red)]">{t('action.killSwitch')}</p>
            <p className="mt-0.5 text-xs text-[var(--text-secondary)]">{copy.killHint}</p>
          </div>
        </div>
      </div>
    </header>
  )
}
