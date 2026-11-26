import { ChevronDown, Moon, OctagonAlert, Sun } from 'lucide-react'
import { useI18n, type LocaleCode } from '../i18n'

interface TopBarProps {
  demo: boolean
  theme: 'dark' | 'light'
  onToggleTheme: () => void
  onEmergencyStop: () => Promise<void>
}

export function TopBar({ demo, theme, onToggleTheme, onEmergencyStop }: TopBarProps) {
  const { locale, setLocale, t } = useI18n()

  return (
    <header className="sticky top-0 z-20 flex flex-wrap items-center justify-between gap-2 border-b border-[var(--border-subtle)] bg-[var(--bg-canvas)]/90 px-3 py-2 backdrop-blur-md sm:px-4">
      <div className="flex min-w-0 items-center gap-2">
        <span className="inline-flex min-h-8 items-center gap-1.5 rounded-full border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-2.5 font-mono text-xs text-[var(--text-secondary)] sm:px-3">
          <span className="h-2 w-2 shrink-0 rounded-full bg-[var(--accent-green)]" />
          <span className="truncate">{demo ? t('status.demo') : t('status.online')}</span>
        </span>
      </div>

      <div className="flex items-center gap-2">
        <div className="relative">
          <select
            aria-label="language"
            value={locale}
            onChange={(e) => setLocale(e.target.value as LocaleCode)}
            className="loom-card min-h-11 appearance-none rounded-xl pr-7 pl-2.5 font-mono text-xs text-[var(--text-primary)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
          >
            <option value="es">ES</option>
            <option value="en">EN</option>
            <option value="pt-BR">PT-BR</option>
          </select>
          <ChevronDown
            className="pointer-events-none absolute top-1/2 right-2 h-3.5 w-3.5 -translate-y-1/2 text-[var(--text-muted)]"
            aria-hidden="true"
          />
        </div>

        <button
          type="button"
          onClick={onToggleTheme}
          aria-label={theme === 'dark' ? t('theme.light') : t('theme.dark')}
          className="loom-card inline-flex min-h-11 min-w-11 items-center justify-center rounded-xl text-[var(--text-primary)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
        >
          {theme === 'dark' ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
        </button>

        <button
          type="button"
          onClick={() => void onEmergencyStop()}
          aria-label={t('action.killSwitch')}
          className="inline-flex min-h-11 items-center gap-1.5 rounded-xl border border-[var(--accent-red)] bg-[var(--accent-red-soft)] px-2.5 text-xs font-medium text-[var(--accent-red)] focus-visible:outline-2 focus-visible:outline-[var(--accent-red)] sm:px-3"
        >
          <OctagonAlert className="h-4 w-4 shrink-0" aria-hidden="true" />
          <span className="sm:hidden">{t('action.killShort')}</span>
          <span className="hidden sm:inline">{t('action.killSwitch')}</span>
        </button>
      </div>
    </header>
  )
}
