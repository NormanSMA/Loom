import { Moon, OctagonAlert, Sun } from 'lucide-react'
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
    <header className="flex flex-wrap items-center justify-between gap-2 border-b border-[var(--border-subtle)] px-4 py-2.5">
      <div className="flex items-center gap-2">
        <span className="inline-flex min-h-8 items-center gap-1.5 rounded-full border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-3 font-mono text-xs text-[var(--text-secondary)]">
          <span className="h-2 w-2 rounded-full bg-[var(--accent-green)]" />
          {demo ? t('status.demo') : t('status.online')}
        </span>
      </div>

      <div className="flex items-center gap-2">
        <select
          aria-label="language"
          value={locale}
          onChange={(e) => setLocale(e.target.value as LocaleCode)}
          className="loom-card min-h-11 rounded-xl px-2.5 font-mono text-xs text-[var(--text-primary)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
        >
          <option value="es">ES</option>
          <option value="en">EN</option>
          <option value="pt-BR">PT-BR</option>
        </select>

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
          className="inline-flex min-h-11 items-center gap-1.5 rounded-xl border border-[var(--accent-red)] bg-[var(--accent-red-soft)] px-3 text-xs font-medium text-[var(--accent-red)] focus-visible:outline-2 focus-visible:outline-[var(--accent-red)]"
        >
          <OctagonAlert className="h-4 w-4" aria-hidden="true" />
          <span>{t('action.killSwitch')}</span>
        </button>
      </div>
    </header>
  )
}
